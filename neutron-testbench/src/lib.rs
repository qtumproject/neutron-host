extern crate neutron_star_constants;
use crate::hypervisor::*;
use crate::interface::*;
use qx86::vm::*;
use neutron_star_constants::*;
use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;

#[derive(Clone, Debug, Default)]
pub struct TestbenchAPI{
    sccs: Vec<Vec<u8>>,
    pub context: NeutronContext
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



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    
}
 