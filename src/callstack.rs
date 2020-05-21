use crate::interface::*;
use crate::addressing::*;

#[derive(Default)]
pub struct ContractCallStack{
    data_stack: Vec<Vec<u8>>,
    context_stack: Vec<ExecutionContext>
}

impl ContractCallStack{
	/// Pushes an item to the Smart Contract Communication Stack
	pub fn push_sccs(&mut self, data: &[u8]) -> Result<(), NeutronError>{
        if data.len() > 0xFFFF{
            return Err(NeutronError::RecoverableFailure);
        }
        self.data_stack.push(data.to_vec());
        Ok(())
    }
    /// Pops an item off of the Smart Contract Communication Stack
	pub fn pop_sccs(&mut self) -> Result<Vec<u8>, NeutronError>{
        match self.data_stack.pop(){
            None => {
                return Err(NeutronError::RecoverableFailure);
            },
            Some(v) => {
                return Ok(v);
            }
        }
    }
    /// Pops an item off of the Smart Contract Communication Stack
	pub fn drop_sccs(&mut self) -> Result<(), NeutronError>{
        if self.data_stack.len() == 0{
            return Err(NeutronError::RecoverableFailure);
        }
        self.data_stack.pop();
        Ok(())
    }
	/// Retrieves the top item on the Smart Contract Communication Stack without removing it
	pub fn peek_sccs(&self, index: u32) -> Result<Vec<u8>, NeutronError>{
        let i = (self.data_stack.len() as isize - 1) - index as isize;
        if i < 0{
            return Err(NeutronError::RecoverableFailure);
        }
        match self.data_stack.get(i as usize){
            None => {
                return Err(NeutronError::RecoverableFailure);
            },
            Some(v) => {
                return Ok(v.to_vec());
            }
        }
    }
	/// Checks the size of the top item on the Smart Contract Communication Stack
    //fn peek_sccs_size(&mut self) -> Result<usize, NeutronError>;
    /// Swaps the top item of the SCCS with the item of the desired index
    /* TODO later
    pub fn sccs_swap(&mut self,index: u32) -> Result<(), NeutronError>{
        Ok(())
    }
    /// Replicates the desired item of the stack onto the top of the stack
    pub fn sccs_dup(&mut self, index: u32) -> Result<(), NeutronError>{
        Ok(())
    }
    */
    /// Gets number of items in the sccs
    pub fn sccs_item_count(&self) -> Result<u32, NeutronError>{
        Ok(self.data_stack.len() as u32)
    }
    /// Get total memory occupied by the SCCS
    /*
    pub fn sccs_memory_amount(&self) -> Result<u32, NeutronError>{
        Ok(0)
    }
    */
    pub fn push_context(&mut self, context: ExecutionContext) -> Result<(), NeutronError>{
        self.context_stack.push(context);
        Ok(())
    }
    pub fn pop_context(&mut self) -> Result<ExecutionContext, NeutronError>{
        match self.context_stack.pop(){
            None => {
                return Err(NeutronError::RecoverableFailure);
            },
            Some(v) => {
                return Ok(v);
            }
        }
    }
    pub fn peek_context(&self, index: usize) -> Result<&ExecutionContext, NeutronError>{
        let i = (self.context_stack.len() as isize - 1) - index as isize;
        if i < 0{
            return Err(NeutronError::RecoverableFailure);
        }
        match self.context_stack.get(i as usize){
            None => {
                return Err(NeutronError::RecoverableFailure);
            },
            Some(v) => {
                return Ok(v);
            }
        }
    }
    pub fn context_count(&self) -> Result<usize, NeutronError>{
        Ok(self.context_stack.len())
    }
	/// Retrieves the context information of the current smart contract execution
	pub fn current_context(&self) -> &ExecutionContext{
        //this should never error, so just unwrap
        self.peek_context(0).unwrap()
    }
    pub fn create_call(&mut self, address: NeutronAddress, sender: NeutronAddress, gas_limit: u64, value: u64){
        let mut c = ExecutionContext::default();
        c.self_address = address.clone();
        c.gas_limit = gas_limit;
        c.value_sent = value;
        c.sender = sender.clone();
        if self.context_stack.len() == 0 {
            c.origin = sender.clone();
        }else{
            c.origin = self.peek_context(0).unwrap().sender.clone();
        }
        c.execution_type = ExecutionType::Call;
        self.push_context(c).unwrap();
    }
}
