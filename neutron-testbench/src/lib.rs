extern crate neutron_star_constants;
use neutron_host::hypervisor::*;
use neutron_host::interface::*;
use neutron_host::db::*;
use qx86::vm::*;
use neutron_star_constants::*;  
use num_traits::FromPrimitive;
use std::collections::HashMap;



#[derive(Clone, Default)]
pub struct TestbenchAPI <'a>{
    sccs: Vec<Vec<u8>>,
	pub context: NeutronContext,
	pub chain: SimulatedBlockchain<'a>,
	pub db: ProtoDB,
}

impl NeutronAPI for TestbenchAPI{
    fn get_context(&self) -> &NeutronContext{
        &self.context
    }
    fn push_sccs(&mut self, data: &Vec<u8>) -> Result<(), NeutronError>{
        self.sccs.push(data.clone());
        Ok(())
    }
    fn pop_sccs(&mut self, data: &mut Vec<u8>) -> Result<(), NeutronError>{
        let p = self.sccs.pop().ok_or(NeutronError::RecoverableFailure)?;
        data.resize(p.len(), 0);
        data.copy_from_slice(&p);
        Ok(())
    }
    fn pop_sccs_toss(&mut self) -> Result<(), NeutronError>{
        if self.sccs.len() == 0{
            Err(NeutronError::RecoverableFailure)
        }else{
            let _ = self.sccs.remove(self.sccs.len() - 1);
            Ok(())
        }
    }
    fn peek_sccs(&mut self, data: &mut Vec<u8>) -> Result<(), NeutronError>{
        if self.sccs.len() == 0{
            Err(NeutronError::RecoverableFailure)
        }else{
            let p = &self.sccs[self.sccs.len() - 1];
            data.copy_from_slice(p);
            Ok(())
        }
    }
    fn peek_sccs_size(&mut self) -> Result<usize, NeutronError>{
        Ok(self.sccs.len())
	}
	
	fn load_state(&mut self, address: NeutronAddress, key: &[u8], data: &mut Vec<u8>) -> Result<usize, NeutronError> {
		// if key does not exist, throw an error
		Ok(0)
	}

	fn store_state(&mut self, address: NeutronAddress, key: &[u8], data: &[u8]) -> Result<(), NeutronError> {
		// if key || value exceeds size limits, throw an error
		Ok(())
	}

	fn load_protected_state(&mut self, address: NeutronAddress, key: &[u8], data: &mut Vec<u8>) -> Result<usize, NeutronError> {
		// if key does not exist, throw an error
		Ok(0)
	}

	fn store_protected_state(&mut self, address: NeutronAddress, key: &[u8], data: &[u8]) -> Result<(), NeutronError> {
		// if key || value exceeds size limits, throw an error
		Ok(())
	}

    fn log_error(&mut self, msg: &str){
        println!("ERROR: {}", msg);
    }
    fn log_info(&mut self, msg: &str){
        println!("INFO: {}", msg);
    }
    fn log_debug(&mut self, msg: &str){
        println!("DEBUG: {}", msg);
    }
}



impl NeutronHypervisor for TestbenchAPI{}
impl Hypervisor for TestbenchAPI{
    fn interrupt(&mut self, vm: &mut VM, num: u8) -> Result<(), VMError>{
        use TestbenchSyscalls::*;
        if num == NEUTRON_INTERRUPT || num == EXIT_INTERRUPT{
            return (self as &mut dyn NeutronHypervisor).interrupt(vm, num);
        }

        if num != TESTBENCH_INTERRUPT{
            self.log_error("Invalid interrupt triggered");
            return Ok(());
        }
        let syscall:TestbenchSyscalls =  FromPrimitive::from_u32(vm.reg32(Reg32::EAX)).unwrap_or(TestbenchSyscalls::Invalid);
        match syscall{
            LogError => {
                //(char *msg, uint32 msg_size) -> void
                let size = vm.reg32(Reg32::ECX);
                let msg = String::from_utf8_lossy(vm.copy_from_memory(vm.reg32(Reg32::EBX), size)?).to_owned();
                self.log_error(&msg);
            },
            LogInfo => {
                //(char *msg, uint32 msg_size) -> void
                let size = vm.reg32(Reg32::ECX);
                let msg = String::from_utf8_lossy(vm.copy_from_memory(vm.reg32(Reg32::EBX), size)?).to_owned();
                self.log_info(&msg);;
            },
            LogDebug => {
                //(char *msg, uint32 msg_size) -> void
                let size = vm.reg32(Reg32::ECX);
                let msg = String::from_utf8_lossy(vm.copy_from_memory(vm.reg32(Reg32::EBX), size)?).to_owned();
                self.log_debug(&msg);
            },
            Invalid => {
                self.log_error("Invalid testbench system call");
            }
            _ => unimplemented!()
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
struct SimulatedBlockchain <'a> {
	pub blocks: Vec<Block>,
	pub contracts: HashMap<String, &'a Contract<'a>>,
}

#[derive(Clone, Debug)]
struct Contract<'a> {
	pub data_section: &'a[String],
	pub code_section: &'a[String],
	pub section_info: [u8; 2],
	pub vm_opts: VMOptions,
}

#[derive(Clone, Debug)]
struct VMOptions {

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
 
