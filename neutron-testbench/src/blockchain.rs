extern crate neutron_star_constants;
use neutron_host::addressing::*;
use neutron_star_constants::*;

#[derive(Clone, Debug, Default)]
struct SimulatedBlockchain  {
	pub blocks: Vec<Block>,
	pub contracts: HashMap<String, Contract>,
	pub balances: HashMap<NeutronAddress, u64>,
	pub gas_limit: u64,
}

impl TestWallet for NeutronAddress {
	/// seals the blockchain and mints a block, reward goes to address
	pub fn seal_block(&mut chain: SimulatedBlockchain) {

	}

	pub fn send_tx(address: NeutronAddress, amount: u64, context: NeutronContext) -> Result<(), NeutronError> {

	}
}

#[derive(Clone, Debug)]
struct Contract {
	pub data_section: [String],
	pub code_section: [String],
	pub section_info: [u8; 2],
	pub vm_opts: VMOptions,
}

#[derive(Clone, Debug)]
struct VMOptions {
    pub version: u8,
}

//type OutputSize = U32

#[derive(Clone, Debug)]
struct Block {
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