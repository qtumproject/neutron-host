extern crate neutron_star_constants;
use neutron_host::addressing::*;
use neutron_host::interface::*;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct SimulatedBlockchain  {
	pub blocks: Vec<Block>,
	pub contracts: HashMap<String, Contract>,
	pub balances: HashMap<NeutronAddress, u64>,
	pub gas_limit: u64,
}

/// test wallet simulates a blockchain environment using one wallet with a multitude of addresses
struct TestWallet {
	pub addresses: Vec<NeutronAddress>
}

impl TestWallet {
	/// seals the blockchain and mints a block, reward goes to address
	fn seal_block(self, chain: &mut SimulatedBlockchain) {

	}

	fn send_tx(address: NeutronAddress, amount: u64, context: NeutronContext) -> Result<(), NeutronError> {
		Ok(())
	}
}

#[derive(Clone, Debug)]
pub struct Contract {
	pub data_section: Box<[String]>,
	pub code_section: Box<[String]>,
	pub section_info: [u8; 2],
	pub vm_opts: VMOptions,
}

#[derive(Clone, Debug)]
pub struct VMOptions {
    pub version: u8,
}

//type OutputSize = U32

#[derive(Clone, Debug)]
pub struct Block {
	pub hash_prev_block: String, // for now this is easy for display
	pub hash_merkle_root: String,
	pub hash_state_root: String,
    pub hash_utxo_root: String,
    pub time: u32,
    pub bits: u32,
    pub nonce: u32,
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    
}