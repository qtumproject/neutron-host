

pub trait MyContract {
	#[neutronify]
    pub fn my_func(&self, x: u8, y: String) -> u32 {}
    pub fn empty_func(&self) -> () {}
}
