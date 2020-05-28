extern crate neutron_star_constants;
use ring::digest::{Context, SHA256};
use rand::Rng;
use neutron_star_constants::*;

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
	
	pub fn set_to_random_address(&mut self) {
		self.data = rand::thread_rng().gen::<[u8; 32]>().to_vec();
    }
    pub fn new_random_address() -> NeutronAddress{
        let mut a = NeutronAddress::default();
        a.set_to_random_address();
        a
    }
}