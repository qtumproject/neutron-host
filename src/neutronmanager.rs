use crate::interface::*;
use crate::addressing::*;
use crate::neutronerror::*;
use crate::neutronerror::NeutronError::*;
use std::collections::HashMap;


#[derive(Default)]
pub struct NeutronManager{
    context_stack: Vec<ExecutionContext>,
    stacks: [Vec<Vec<u8>>; 2],
    maps: Vec<HashMap<Vec<u8>, Vec<u8>>>,
    input_stack: usize,
    output_stack: usize,
    top_input_map: usize,
    top_output_map: usize,
    top_result_map: usize
}

impl NeutronManager{
    pub fn new() -> NeutronManager{
        let mut manager = NeutronManager::default();
        manager.maps.push(HashMap::<Vec<u8>, Vec<u8>>::default()); //add output map (note: this is flipped when context is pushed)
        manager.maps.push(HashMap::<Vec<u8>, Vec<u8>>::default()); //add input map
        manager.top_input_map = 1;
        manager.top_output_map = 0;
        manager.top_result_map = 1;
        manager.input_stack = 0;
        manager.output_stack = 1;
        //manager.stacks.push(Vec::<Vec<u8>>::default());
        manager
    }

    pub fn push_stack(&mut self, data: &[u8]) -> Result<(), NeutronError>{
        self.stacks[self.output_stack].push(data.to_vec());
        Ok(())
    }
	pub fn pop_stack(&mut self) -> Result<Vec<u8>, NeutronError>{
        match self.stacks[self.input_stack].pop(){
            None => {
                return Err(Recoverable(RecoverableError::ItemDoesntExist));
            },
            Some(v) => {
                return Ok(v);
            }
        }
    }
	pub fn drop_stack(&mut self) -> Result<(), NeutronError>{
        match self.stacks[self.input_stack].pop(){
            None => {
                return Err(Recoverable(RecoverableError::ItemDoesntExist));
            },
            Some(v) => {
                return Ok(());
            }
        }
    }
	pub fn peek_stack(&self, index: u32) -> Result<Vec<u8>, NeutronError>{
        let stack = &self.stacks[self.input_stack];
        let i = (stack.len() as isize - 1) - index as isize;
        if i < 0{
            return Err(Recoverable(RecoverableError::ItemDoesntExist));
        }
        match stack.get(i as usize){
            None => {
                return Err(Recoverable(RecoverableError::ItemDoesntExist));
            },
            Some(v) => {
                return Ok(v.to_vec());
            }
        }
    }

    pub fn push_key(&mut self, key: &[u8], value: &[u8]) -> Result<(), NeutronError>{
        self.maps.get_mut(self.top_output_map).unwrap().insert(key.to_vec(), value.to_vec());
        Ok(())
    }
    /* should this be allowed?
    pub fn pop_key(&mut self, key: &[u8]) -> Result<Vec<u8>, NeutronError>{
        match self.maps[self.top_input_map].remove(key){
            Some(v) => {
                Ok(v)
            },
            None => {
                Err(Recoverable(RecoverableError::ItemDoesntExist))
            }
        }
    }
    */
    pub fn peek_key(&self, key: &[u8]) -> Result<Vec<u8>, NeutronError>{
        match self.maps[self.top_input_map].get(key){
            Some(v) => {
                Ok(v.to_vec())
            },
            None => {
                Err(Recoverable(RecoverableError::ItemDoesntExist))
            }
        }
    }
    pub fn peek_result_key(&self, key: &[u8]) -> Result<Vec<u8>, NeutronError>{
        match self.maps[self.top_result_map].get(key){
            Some(v) => {
                Ok(v.to_vec())
            },
            None => {
                Err(Recoverable(RecoverableError::ItemDoesntExist))
            }
        }
    }

    /// Should only be used by Element APIs. Flip stacks once when entering an Element API and once more when leaving and returning to a contract.
    /// Used so that contract outputs become Element inputs at first, then so that Element outputs becomes contract inputs
    fn flip_stacks(&mut self){
        let tmp = self.input_stack;
        self.input_stack = self.output_stack;
        self.output_stack = tmp;
        self.stacks[self.output_stack].clear(); //outputs are cleared with each flipping (clears caller's outputs on entry, then callers inputs upon exit)
    }

    /*
    Map Management
    new state, 1: 3 maps added: inputA1, outputA2, resultA3
    Call initiated, 2: 1 map added, resultB4. input2 points to outputA2, output2 points to resultA3
    Call initiated, 3: 1 map added, resultC5. input3 points to ResultA3, output3 points to ResultB4 
    Call initiated, 4: 1 map added, ResultD5. input4 points to ResultB3, output4 points to ResultC5
    Call ends, 4 destroyed: ResultD destroyed, top input(3) equivalent to (output2)ResultA. top output(3) equivalent to (input4)ResultB, is cleared. top result(3) equivalent to (output4)ResultC
    End result per call: input = len-1, output = len, result = len+1
    */

    /// Pushes a new execution context into the stack
    pub fn push_context(&mut self, context: ExecutionContext) -> Result<(), NeutronError>{
        let mut c = context;
        self.top_input_map = self.top_output_map; //one below top of stack
        self.top_output_map = self.top_result_map; //top of stack
        self.top_result_map = self.top_result_map + 1; //new (temporary) map

        c.input_map = self.top_input_map;
        c.output_map = self.top_output_map;
        c.result_map = self.top_result_map;
        self.maps.get_mut(self.top_output_map).unwrap().clear(); //clear what is now the new result map (which can go on to become the next call's output map)
        self.maps.push(HashMap::<Vec<u8>, Vec<u8>>::new()); //push new result map
        self.context_stack.push(c);
        //begin execution???
        Ok(())
    }

    /// Note for Element functions which result in a context push (ie, a call), 
    /// enter and exit functions should both be used before pushing the context and starting the call (and beginning execution),
    /// and then both again used after popping the context and exiting the call (returning execution to caller)
    pub fn enter_element(&mut self){
        self.top_input_map = self.top_output_map; //one below top of stack
        self.top_output_map = self.top_result_map; //top of stack
        self.flip_stacks();
        //note: elements can not access result map 
    }
    pub fn exit_element(&mut self){
        let c = self.context_stack.last().unwrap();
        self.top_output_map = c.output_map;
        self.top_input_map = c.input_map;
        self.flip_stacks();
    }
    /// Removes the top execution context from the stack
    pub fn pop_context(&mut self) -> Result<(), NeutronError>{
        match self.context_stack.pop(){
            None => {
                return Err(Unrecoverable(UnrecoverableError::ContextIndexEmpty));
            },
            Some(v) => {}
        }
        let c = match self.context_stack.last(){
            None => {
                //no more contexts, so set to transaction level behavior
                self.top_input_map = 1;
                self.top_output_map = 0;
                self.top_result_map = 1;
                return Ok(());
            },
            Some(v) => {v}
        };
        self.maps.pop().unwrap(); //result map of caller is destroyed
        self.top_input_map = c.input_map;
        self.top_output_map = c.output_map;
        self.top_result_map = c.result_map;
        Ok(())
    }
    /// Peeks information from the execution context stack without modifying it
    pub fn peek_context(&self, index: usize) -> Result<&ExecutionContext, NeutronError>{
        let i = (self.context_stack.len() as isize - 1) - index as isize;
        if i < 0{
            return Err(Recoverable(RecoverableError::ItemDoesntExist));
        }
        match self.context_stack.get(i as usize){
            None => {
                return Err(Recoverable(RecoverableError::ItemDoesntExist));
            },
            Some(v) => {
                return Ok(v);
            }
        }
    }
    /// The total number of smart contract contexts currently involved in the overall execution
    pub fn context_count(&self) -> Result<usize, NeutronError>{
        Ok(self.context_stack.len())
    }

	/// Retrieves the context information of the current smart contract execution
	pub fn current_context(&self) -> &ExecutionContext{
        //this should never error, so just unwrap
        self.peek_context(0).unwrap()
    }

    /// Creates a top level context for calling an existing contract. The context stack MUST be empty
    pub fn create_top_level_call(&mut self, address: NeutronAddress, sender: NeutronAddress, gas_limit: u64, value: u64){
        assert!(self.context_stack.len() == 0);
        let mut c = ExecutionContext::default();
        c.self_address = address.clone();
        c.gas_limit = gas_limit;
        c.value_sent = value;
        c.sender = sender.clone();
        c.origin = sender.clone();
        c.execution_type = ExecutionType::Call;
        self.push_context(c).unwrap();
    }
    /// Creates a top level context for deploying a new contract. The context stack MUST be empty
    pub fn create_top_level_deploy(&mut self, address: NeutronAddress, sender: NeutronAddress, gas_limit: u64, value: u64){
        assert!(self.context_stack.len() == 0);
        //todo: dedupicate
        let mut c = ExecutionContext::default();
        c.self_address = address.clone();
        c.gas_limit = gas_limit;
        c.value_sent = value;
        c.sender = sender.clone();
        c.origin = sender.clone();
        c.execution_type = ExecutionType::Deploy;
        self.push_context(c).unwrap();
    }
    /// Creates a new nested context for calling an existing contract. The context stack MUST NOT be empty
    pub fn create_call(&mut self, address: NeutronAddress, gas_limit: u64, value: u64){
        assert!(self.context_stack.len() > 0);
        let mut c = ExecutionContext::default();
        c.self_address = address.clone();
        c.gas_limit = gas_limit;
        c.value_sent = value;
        c.sender = self.peek_context(0).unwrap().self_address.clone();
        c.origin = self.context_stack.get(0).unwrap().sender.clone();
        c.execution_type = ExecutionType::Call;
        self.push_context(c).unwrap();
    }
    /// Creates a new nested context for deploying a contract. The context stack MUST NOT be empty
    pub fn create_deploy(&mut self, address: NeutronAddress, gas_limit: u64, value: u64){
        assert!(self.context_stack.len() > 0);
        let mut c = ExecutionContext::default();
        c.self_address = address.clone();
        c.gas_limit = gas_limit;
        c.value_sent = value;
        c.sender = self.peek_context(0).unwrap().self_address.clone();
        c.origin = self.context_stack.get(0).unwrap().sender.clone();
        c.execution_type = ExecutionType::Deploy;
        self.push_context(c).unwrap();
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_call_map_flow(){
        let mut manager = NeutronManager::new();
        let c1 = ExecutionContext::default();
        let c2 = ExecutionContext::default();
        let c3 = ExecutionContext::default();
        let key = [0];
        //ABI data
        manager.push_key(&key, &[1]).unwrap();
        //call from transaction
        {
            manager.push_context(c1).unwrap();
            manager.push_key(&key, &[2]).unwrap();
            assert_eq!(manager.peek_key(&key).unwrap()[0], 1);
            //call into sub contract
            {
                manager.enter_element();
                manager.exit_element();
                manager.push_context(c2).unwrap();
                manager.push_key(&key, &[3]).unwrap();
                assert_eq!(manager.peek_key(&key).unwrap()[0], 2);
                //call into sub sub contract
                {
                    manager.enter_element();
                    manager.exit_element();
                    manager.push_context(c3).unwrap();
                    manager.push_key(&key, &[4]).unwrap();
                    assert_eq!(manager.peek_key(&key).unwrap()[0], 3);
                    manager.pop_context().unwrap();
                    manager.enter_element();
                    manager.exit_element();
                }
                assert_eq!(manager.peek_result_key(&key).unwrap()[0], 4);
                assert_eq!(manager.peek_key(&key).unwrap()[0], 2);
                manager.pop_context().unwrap();
                manager.enter_element();
                manager.exit_element();
            }
            assert_eq!(manager.peek_result_key(&key).unwrap()[0], 3);
            assert_eq!(manager.peek_key(&key).unwrap()[0], 1);
            manager.pop_context().unwrap();
        }
        
        assert_eq!(manager.peek_key(&key).unwrap()[0], 2);
        assert_eq!(manager.peek_result_key(&key).unwrap()[0], 2);
    }
    #[test]
    fn test_element_stack_flow(){
        let mut manager = NeutronManager::new();
        let c1 = ExecutionContext::default();        
        let key = [0];
        //ABI data
        manager.push_key(&key, &[1]).unwrap();
        //element data
        manager.push_context(c1).unwrap();
        manager.push_stack(&[2]).unwrap();
        manager.push_key(&key, &[5]).unwrap();
        {
            manager.enter_element();
            manager.push_stack(&[3]).unwrap();
            manager.push_key(&key, &[4]).unwrap();
            assert_eq!(manager.peek_key(&key).unwrap()[0], 5);
            assert_eq!(manager.pop_stack().unwrap()[0], 2);
            manager.exit_element();
        }
        assert_eq!(manager.peek_result_key(&key).unwrap()[0], 4);
        assert_eq!(manager.pop_stack().unwrap()[0], 3);
        manager.pop_context().unwrap();
    }
}