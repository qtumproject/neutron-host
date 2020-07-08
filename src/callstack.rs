extern crate qx86;

use crate::interface::*;
use crate::addressing::*;
use crate::neutronerror::*;
use crate::neutronerror::NeutronError::*;
use qx86::vm::*;


pub trait GasSchedule{
    fn x86_gas_schedule(&self) -> GasCharger;
    fn gas_cost(&self, feature: u32, costid: u32) -> i64;
}

pub struct BlankSchedule{}
impl GasSchedule for BlankSchedule{
    fn x86_gas_schedule(&self) -> GasCharger{
        GasCharger::test_schedule()
    }
    fn gas_cost(&self, _feature: u32, _costid: u32) -> i64{
        0
    }
}

pub const INTERNAL_BUILT_IN_FEATURE:u32 = 0;

pub enum CallStackCost{
    WriteByte = 1,
    ReadByte,
    ClearByteRefund,
    CopyDataToVM, //not used here, but used in hypervisors
    CopyDataFromVM
}

fn gas_cost(stack: &ContractCallStack, costid: CallStackCost) -> i64{
    stack.gas_cost(INTERNAL_BUILT_IN_FEATURE, costid as u32)
}

/// The primary call stack which is used for almost all communication purposes between the system call layer and VMs
/// It contains context information for the current smart contracts being executed and a shared general purpose stack
/// All smart contract VMs should use this structure for all communication purposes with "the outside world"
pub struct ContractCallStack{
    data_stack: Vec<Vec<u8>>,
    context_stack: Vec<ExecutionContext>,
    /// Note these fields are primary used for communication between the CallSystem, Hypervisor, and VM. 
    pub pending_gas: i64,
    /// Note these fields are primary used for communication between the CallSystem, Hypervisor, and VM. 
    pub gas_remaining: u64,
    gas_schedule: Box<dyn GasSchedule>,
    system_charges_enabled: bool
}

impl Default for ContractCallStack{
    fn default() -> ContractCallStack{
        ContractCallStack{
            data_stack: Vec::default(),
            context_stack: Vec::default(),
            pending_gas: 0,
            gas_remaining: 0,
            gas_schedule: Box::from(BlankSchedule{}),
            system_charges_enabled: true
        }
    }
}

impl ContractCallStack{
    pub fn new_with_schedule(schedule: Box<dyn GasSchedule>) -> ContractCallStack{
        ContractCallStack{
            data_stack: Vec::default(),
            context_stack: Vec::default(),
            pending_gas: 0,
            gas_remaining: 0,
            gas_schedule: schedule,
            system_charges_enabled: true
        }
    }
    pub fn disable_system_charges(&mut self){
        self.system_charges_enabled = false;
    }
    pub fn enable_system_charges(&mut self){
        self.system_charges_enabled = true;
    }
    pub fn x86_gas_charger(&self) -> GasCharger{
        self.gas_schedule.x86_gas_schedule()
    }
    pub fn gas_cost(&self, feature: u32, costid: u32) -> i64{
        if self.system_charges_enabled {
            self.gas_schedule.gas_cost(feature, costid)
        }else{
            0
        }
    }
    /// Adds to the current amount of gas consumed by the system call, and returns a recoverable error if there is not enough gas to satisfy it
    pub fn charge_gas(&mut self, amount: i64) -> Result<(), NeutronError>{
        self.pending_gas += amount;
        if self.pending_gas > self.gas_remaining as i64{
            return Err(NeutronError::Recoverable(RecoverableError::OutOfGas));
        }
        Ok(())
    }
	/// Pushes an item to the Smart Contract Communication Stack
	pub fn push_sccs(&mut self, data: &[u8]) -> Result<(), NeutronError>{
        if data.len() > 0xFFFF{
            return Err(Recoverable(RecoverableError::StackItemTooLarge));
        }
        self.charge_gas(gas_cost(self, CallStackCost::WriteByte) * data.len() as i64)?;
        self.data_stack.push(data.to_vec());
        Ok(())
    }
    /// Pops an item off of the Smart Contract Communication Stack
	pub fn pop_sccs(&mut self) -> Result<Vec<u8>, NeutronError>{
        match self.data_stack.pop(){
            None => {
                return Err(Recoverable(RecoverableError::StackIndexDoesntExist));
            },
            Some(v) => {
                let cost = gas_cost(self, CallStackCost::ReadByte) * v.len() as i64
                         - gas_cost(self, CallStackCost::ClearByteRefund) * v.len() as i64;
                self.charge_gas(cost)?;
                return Ok(v);
            }
        }
    }
    /// Pops an item off of the Smart Contract Communication Stack
	pub fn drop_sccs(&mut self) -> Result<(), NeutronError>{
        if self.data_stack.len() == 0{
            return Err(Recoverable(RecoverableError::StackIndexDoesntExist));
        }
        let data = self.data_stack.pop().unwrap();
        let cost = gas_cost(self, CallStackCost::ClearByteRefund) * data.len() as i64;
        self.charge_gas(cost)?;
        Ok(())
    }
	/// Retrieves the top item on the Smart Contract Communication Stack without removing it
	pub fn peek_sccs(&mut self, index: u32) -> Result<Vec<u8>, NeutronError>{
        let i = (self.data_stack.len() as isize - 1) - index as isize;
        if i < 0{
            return Err(Recoverable(RecoverableError::StackIndexDoesntExist));
        }
        match self.data_stack.get(i as usize){
            None => {
                return Err(Recoverable(RecoverableError::StackIndexDoesntExist));
            },
            Some(v) => {
                let data = v.to_vec();
                let cost = gas_cost(self, CallStackCost::ReadByte) * data.len() as i64;
                self.charge_gas(cost)?;
                return Ok(data);
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

    /// Pushes a new execution context into the stack
    pub fn push_context(&mut self, context: ExecutionContext) -> Result<(), NeutronError>{
        self.context_stack.push(context);
        Ok(())
    }
    /// Removes the top execution context from the stack
    pub fn pop_context(&mut self) -> Result<ExecutionContext, NeutronError>{
        match self.context_stack.pop(){
            None => {
                return Err(Unrecoverable(UnrecoverableError::ContextIndexEmpty));
            },
            Some(v) => {
                return Ok(v);
            }
        }
    }
    /// Peeks information from the execution context stack without modifying it
    pub fn peek_context(&self, index: usize) -> Result<&ExecutionContext, NeutronError>{
        let i = (self.context_stack.len() as isize - 1) - index as isize;
        if i < 0{
            return Err(Recoverable(RecoverableError::StackIndexDoesntExist));
        }
        match self.context_stack.get(i as usize){
            None => {
                return Err(Recoverable(RecoverableError::StackIndexDoesntExist));
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
