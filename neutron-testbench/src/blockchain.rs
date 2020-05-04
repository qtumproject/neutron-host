extern crate neutron_star_constants;
use neutron_host::addressing::*;
use neutron_host::interface::*;
use neutron_host::db::*;
use std::collections::HashMap;
use std::str;
use std::string::*;

#[derive(Clone, Debug, Default)]
pub struct SimulatedBlockchain  {
	pub contracts: HashMap<String, Contract>,
	pub state: ProtoDB,
	pub balances: HashMap<NeutronAddress, u64>,
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    
}