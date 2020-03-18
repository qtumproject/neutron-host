extern crate neutron_star_constants;
extern crate ring;
extern crate struct_deser;
#[macro_use]
use struct_deser_derive::*;
use neutron_star_constants::*;
use ring::digest::{Context, SHA256};

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct NeutronAddress{
    pub version: u32,
    pub data: Vec<u8>
}

impl NeutronAddress{
    pub fn to_short_address(&self) -> NeutronShortAddress{
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
}




#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct NeutronVMResult{
    pub gas_used: u64,
    pub should_revert: bool,
    pub error_code: u32,
    pub error_location: u64,
    pub extra_data: u64
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct NeutronContext{
    pub exec: ExecContext,
    pub tx: TransactionContext,
    pub block: BlockContext,
    pub internal: usize
}



#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ExecContext{
    pub flags: u64,
    pub sender: NeutronAddress,
    pub gas_limit: u64,
    pub value_sent: u64,
    pub origin: NeutronAddress,
    pub self_address: NeutronAddress,
    pub nest_level: u32
}

impl ExecContext{
    pub fn to_neutron(&self) -> NeutronExecContext{
        let mut c = NeutronExecContext::default();
        c.flags = self.flags;
        c.gas_limit = self.gas_limit;
        c.nest_level = self.nest_level;
        c.value_sent = self.value_sent;
        c
    }
}


#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct TransactionContext{
    pub inputs: Vec<TxItem>,
    pub outputs: Vec<TxItem>
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct TxItem{
    pub sender: NeutronAddress,
    pub value: u64
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct BlockContext{
    pub creator: NeutronAddress,
    pub gas_limit: u64,
    pub difficulty: u64,
    pub height: u32,
    pub previous_time: u64,
    pub previous_hashes: Vec<[u8; 32]>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NeutronError{
    Success,
    RecoverableFailure,
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
    fn get_context(&self) -> &NeutronContext;
    fn push_sccs(&mut self, data: &[u8]) -> Result<(), NeutronError>;
    fn pop_sccs(&mut self, data: &mut Vec<u8>) -> Result<(), NeutronError>;
    fn pop_sccs_toss(&mut self) -> Result<(), NeutronError>; //returns no data, for throwing away the item
    fn peek_sccs(&mut self, data: &mut Vec<u8>) -> Result<(), NeutronError>;
    fn peek_sccs_size(&mut self) -> Result<usize, NeutronError>;

    fn log_error(&mut self, msg: &str);
    fn log_info(&mut self, msg: &str);
    fn log_debug(&mut self, msg: &str);
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
 