extern crate neutron_star_constants;
extern crate ring;
extern crate struct_deser;
#[macro_use]
use struct_deser_derive::*;
use neutron_star_constants::*;
use ring::digest::{Context, SHA256};
use rand::Rng;

#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
/// NeutronAddress is a full dynamic length address. 
/// Due to it being dynamic length it is inconvenient to use, but is required for sending coins to an address
pub struct NeutronAddress{
	/// The type of address
	pub version: u32,
	/// The unique data for the specified address
    pub data: Vec<u8>
}

impl NeutronAddress{
	/// Converts a full address into a short address
    pub fn to_short_address(&self) -> NeutronShortAddress{
		// this needs an exception for data sizes equal to or smaller than 20 bytes??
        // ie, a contract address should be valid as a short address and full address because it's data is always 20 bytes
        let mut context = Context::new(&SHA256);
        context.update(&self.data[0..]);
        let d = context.finish();
        let mut data: [u8; 20] = Default::default();
        data.copy_from_slice(&d.as_ref()[0..20]);
        NeutronShortAddress{
            version: self.version,
            data: data
        }
	}
	
	pub fn generate_random_address(&mut self) {
		self.data = rand::thread_rng().gen::<[u8; 32]>().to_vec();
	}
}



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

/// The context under which a smart contract is being executed
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct NeutronContext{
	/// The execution context, containing info specific to this exact execution
	pub exec: ExecContext,
	/// The transaction context, containing info specific to the transaction within which this execution has occurred
	pub tx: TransactionContext,
	/// The block context, containing info specific to the entire block into which this execution is taking place
	pub block: BlockContext,
    /// The amount of gas remaining for the current smart contract execution
    /// This is updated implicitly by calls to NeutronAPI functions and is expected to be updated by the VM for computation requests 
    pub gas: u64,
    /// An internal undefined field which can be used to store a pointer.
    pub internal: usize
}


/// The execution context of the current smart contract
/// Multiple ExecContext structs are expected, a new one for each smart contract call performed. 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ExecContext{
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
	/// The total call stack depth of the smart contract executions. 
    /// This is 0 when executed from the transaction and increases by one with each smart contract call and decreases by one with each end of smart contract execution
    pub nest_level: u32
}

impl ExecContext{
	/// ???
    pub fn to_neutron(&self) -> NeutronExecContext{
        let mut c = NeutronExecContext::default();
        c.flags = self.flags;
        c.gas_limit = self.gas_limit;
        c.nest_level = self.nest_level;
        c.value_sent = self.value_sent;
        c
    }
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
    pub value: u64
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

/// The primary error structure of NeutronAPI calls
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NeutronError{
	/// Success, no error has occured
	Success,
	/// An error has occured, but if the VM implements an error handling system, it is appropriate to allow this error
    /// to be handled by the smart contract and for execution to continue
	RecoverableFailure,
    /// An error has occured and the VM should immediately terminate, not allowing the smart contract to detect or handle this error in any capacity
    UnrecoverableFailure
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
#[derive(StructDeser, Debug, Eq, PartialEq, Default)]
pub struct NeutronVersion{
    pub format: u8,
    pub root_vm: u8,
    pub vm_version: u8,
    #[le]
    pub flags: u16,
    #[le]
    pub qtum_version: u32
}


/// This is the primary NeutronAPI interface. It is loosely based on the C Neutron API, but uses Rust paradigms and features
/// This will require a heavier translation layer, but makes Rust usage significantly simpler
pub trait NeutronAPI{
	/// Retrieves the context information of the current smart contract execution
	fn get_context(&self) -> &NeutronContext;
	/// Pushes an item to the Smart Contract Communication Stack
	fn push_sccs(&mut self, data: &Vec<u8>) -> Result<(), NeutronError>;
    /// Pops an item off of the Smart Contract Communication Stack
	fn pop_sccs(&mut self, data: &mut Vec<u8>) -> Result<(), NeutronError>;
	/// Drops an item from the Smart Contract Communication Stack, popping it and doing nothing with the data
	fn pop_sccs_toss(&mut self) -> Result<(), NeutronError>; //returns no data, for throwing away the item
	/// Retrieves the top item on the Smart Contract Communication Stack without removing it
	fn peek_sccs(&mut self, data: &mut Vec<u8>) -> Result<(), NeutronError>;
	/// Checks the size of the top item on the Smart Contract Communication Stack
    fn peek_sccs_size(&mut self) -> Result<usize, NeutronError>;
	
	/// Loads user accessible state from the smart contract database
    fn load_state(&mut self, address: String, key: String, data: &mut String) -> Result<usize, NeutronError>;
    /// Writes user accessible state to the smart contract database
    fn store_state(&mut self, address: String, key: String, data: &mut String) -> Result<(), NeutronError>;
    /// Loads "protected" state from the smart contract database. Protected state can include bytecode, VM configuration options, etc. 
    /// Protected state should not be freely exposed to smart contracts 
    fn load_protected_state(&mut self, address: String, key: String, data: &mut String) -> Result<usize, NeutronError>;
    /// Writes "protected" state to the smart contract database. Protected state can include bytecode, VM configuration options, etc. 
    /// Protected state should not be freely exposed to smart contracts 
    fn store_protected_state(&mut self, address: String, key: String, data: &mut String) -> Result<(), NeutronError>;
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
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
 