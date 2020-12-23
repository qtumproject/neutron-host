use crate::addressing::*;
use crate::callstack::*;
use crate::neutronerror::*;


/// The result of a smart contract execution
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct NeutronVMResult{
	/// The total amount of gas used by the execution
	pub gas_used: u64,
	/// If set to true, then no state effects should've occured from this execution and any state effects should be reverted
	pub should_revert: bool,
	/// The error code specifying how this contract ended
	pub error_code: u32,
	/// An undefined ID of the location of the contract error (for x86 this is the 'EIP' register)
	pub error_location: u64,
	/// Extra data which a smart contract VM is free to use. This is not exposed to smart contracts
    pub extra_data: u64
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionType{
    Call = 0,
    Deploy,
    BareExecution
}

impl Default for ExecutionType{
    fn default() -> ExecutionType{
        ExecutionType::Call
    }
}

/// The execution context of the current smart contract
/// Multiple ExecContext structs are expected, a new one for each smart contract call performed. 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ExecutionContext{
	/// TBD
	pub flags: u64,
    /// The address which caused this execution to occur.
    /// This may be the sender of the transaction, or the smart contract which caused this execution to occur via a call.
	pub sender: NeutronAddress,
    /// The total amount of gas allowed to be consumed in this execution
	pub gas_limit: u64,
	/// The number of coins which were sent with this execution
	pub value_sent: u64,
	/// The address which caused this chain of execution to occur.
    /// This is the sender of the transaction which caused this execution.
	pub origin: NeutronAddress,
	/// The current address of the executing smart contract
    pub self_address: NeutronAddress,
    pub execution_type: ExecutionType,
}

impl ExecutionContext{
}



/// The transaction information in which the current contract execution is located
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct TransactionContext{
	/// The spent UTXOs which make up this transaction
	pub inputs: Vec<TxItem>,
	/// The created UTXOs, contract executions, and other misc data which make up this transaction
	pub outputs: Vec<TxItem>,
    /// The total amount of coins spent by gas fees
    /// Note that this only counts for gas_limit, as it can not be known how much actual gas will be consumed until the transaction is complete
    pub total_gas_fees: u64,
    /// The total fee in coins sent with the transaction. This includes the above total_gas_fees and also any other transaction fees. 
    pub total_fees: u64
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct TxItem{
	/// The owner of this UTXO (or spent UTXO)
	pub sender: NeutronAddress,
	/// The total value sent with this UTXO (or spent by it)
    pub value: u64,
    /// The state sent with this UTXO
    pub state: Vec<u8>
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct BlockContext{
	/// The creator of the current block
	pub creator: NeutronAddress,
	/// The total gas limit for the entire block
	pub gas_limit: u64,
	/// The difficulty of the current block (the meaning of this varies by blockchain)
	pub difficulty: u64,
	/// The block height of the current block
	pub height: u32,
    /// The time recorded in the block just before this one (the current time can not be revealed by all blockchains due to determinism problems)
	pub previous_time: u64,
	/// The previous block hashes leading up to this block.
    /// previous_hashes[0] is the previous block, previous_hashes[1] is the block before that, and so on
    /// Not all blockchains will reveal an entire list of block hashes in this field.
	pub previous_hashes: Vec<[u8; 32]>
}



/*
typedef struct{
    uint8_t format;
    uint8_t rootVM;
    uint8_t vmVersion;
    uint16_t flagOptions;
    uint32_t qtumVersion;
} NeutronVersion;
*/
#[derive(Debug, Eq, PartialEq, Default)]
pub struct NeutronVersion{
    pub format: u8,
    pub root_vm: u8,
    pub vm_version: u8,
    pub flags: u16,
    pub qtum_version: u32
}





pub trait VMInterface{
    fn execute(&mut self) -> Result<NeutronVMResult, NeutronError>;
}

pub trait CallSystem{
    /// General system call interface
    fn system_call(&mut self, stack: &mut ContractCallStack, feature: u32, function: u32) -> Result<u32, NeutronError>;
    /// Get the current block height at execution
    /// Used to switch VM behavior in blockchain forks
    fn block_height(&self) -> Result<u32, NeutronError>;
    /// Read a state key from the database using the permanent storage feature set
    /// Used for reading core contract bytecode by VMs
    fn read_state_key(&mut self, stack: &mut ContractCallStack, space: u8, key: &[u8]) -> Result<Vec<u8>, NeutronError>;
    /// Write a state key to the database using the permanent storage feature set
    /// Used for writing bytecode etc by VMs
    fn write_state_key(&mut self, stack: &mut ContractCallStack, space: u8, key: &[u8], value: &[u8]) -> Result<(), NeutronError>;

    fn log_error(&self, msg: &str){
        println!("Error: {}", msg);
    }
    fn log_warning(&self, msg: &str){
        println!("Warning: {}", msg);
    }
    fn log_info(&self, msg: &str){
        println!("Info: {}", msg);
    }
    fn log_debug(&self, msg: &str){
        println!("Debug: {}", msg);
    }
}




 
    /*
    leftovers from NeutronAPI that need to be implemented in system contracts
	/// Loads user accessible state from the smart contract database
    fn load_state(&mut self, address: NeutronAddress, key: &[u8], data: &mut Vec<u8>) -> Result<usize, NeutronError>;
    /// Writes user accessible state to the smart contract database
    fn store_state(&mut self, address: NeutronAddress, key: &[u8], data: &[u8]) -> Result<(), NeutronError>;
    /// Loads "protected" state from the smart contract database. Protected state can include bytecode, VM configuration options, etc. 
    /// Protected state should not be freely exposed to smart contracts 
    fn load_protected_state(&mut self, address: NeutronAddress, key: &[u8], data: &mut Vec<u8>) -> Result<usize, NeutronError>;
    /// Writes "protected" state to the smart contract database. Protected state can include bytecode, VM configuration options, etc. 
    /// Protected state should not be freely exposed to smart contracts 
    fn store_protected_state(&mut self, address: NeutronAddress, key: &[u8], data: &[u8]) -> Result<(), NeutronError>;
    /// Loads user accessible state from another smart contract's "namespace" in the smart contract database.  
    fn load_external_state(&mut self, address: &NeutronShortAddress, key: &[u8], data: &mut Vec<u8>) -> Result<usize, NeutronError>;
    /// Loads "protected" state from the smart contract database which is from another smart contract's namespace. 
    /// Protected state can include bytecode, VM configuration options, etc. Protected state should not be freely exposed to smart contracts 
    fn load_external_protected_state(&mut self, address: &NeutronShortAddress, key: &[u8], data: &mut Vec<u8>) -> Result<usize, NeutronError>;

    /// Transfers coins from the currently executing smart contract to the specified address
    fn transfer(&mut self, address: &NeutronAddress, value: u64) -> Result<(), NeutronError>;
    /// Transfers coins from the currently executing smart contract to the specified address
    /// This can only be used for valid short addresses where the amount of data in a full address exactly matches the size of a short address
    fn transfer_short(&mut self, address: &NeutronShortAddress, value: u64) -> Result<(), NeutronError>;
    /// Returns the balance of the currently executing smart contract
    fn balance(&mut self) -> Result<u64, NeutronError>;
    /// Checks the balance of an external smart contract. This can not be used for checking the balance of non-contract addresses.
    fn balance_of_external(&mut self, address: &NeutronShortAddress) -> Result<u64, NeutronError>;

    /// Gets the block hash of the specified block
    fn get_block_hash(&mut self, number: u64, hash: &mut[u8]) -> Result<(), NeutronError>;

    /// Calculates the difference in gas cost produced by changing the amount of allocated memory.
    /// Note this does not actually allocate any memory, this is left to the specific VM and hypervisor.
    /// This is only for charging an appropriate gas cost to the smart contract for allocating/freeing memory.
    fn calculate_memory_cost(&self, existing_size: u64, new_size: u64) -> Result<i64, NeutronError>;
    /// Calculates the difference in gas cost produced by changing the amount of allocated read-only memory.
    /// Note this does not actually allocate any memory nor charge the smart contract for the gas, this is left to the specific VM and hypervisor.
    /// This is only for charging an appropriate gas cost to the smart contract for allocating/freeing memory.
    fn calculate_readonly_memory_cost(&self, existing_size: u64, new_size: u64) -> Result<i64, NeutronError>;

    /// This is used for charging (or refunding) the smart contract for a specific gas cost, such as memory allocation
    fn add_gas_cost(&mut self, gas_difference: i64) -> Result<u64, NeutronError>;



    /// Logs an error message. Only for diagnostic purposes, does not have any consensus effect and may effectively be a no-op
    fn log_error(&mut self, msg: &str);
    /// Logs an informational message. Only for diagnostic purposes, does not have any consensus effect and may effectively be a no-op
    fn log_info(&mut self, msg: &str);
    /// Logs a debug message. Only for diagnostic purposes, does not have any consensus effect and may effectively be a no-op
    fn log_debug(&mut self, msg: &str);
    */