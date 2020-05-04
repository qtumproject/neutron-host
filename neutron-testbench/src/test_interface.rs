extern crate neutron_star_constants;
use neutron_host::hypervisor::*;
use neutron_host::interface::*;
use neutron_host::db::*;
use neutron_host::addressing::*;
use qx86::vm::*;
use qx86::structs::ValueSize;
use neutron_star_constants::*;  
use num_traits::FromPrimitive;
use crate::blockchain::SimulatedBlockchain;
use std::cmp::max;

#[derive(Default)]
pub struct TestbenchAPI{
    sccs: Vec<Vec<u8>>,
	pub context: NeutronContext,
	pub chain: SimulatedBlockchain,
	pub db: ProtoDB,
}

impl CallSystem for TestbenchAPI{
    fn get_context(&self) -> &NeutronContext{
        &self.context
    }
    fn push_sccs(&mut self, vm: &mut VM, data: &Vec<u8>) -> Result<(), NeutronError>{
        let address = vm.get_reg(Reg32::ECX as u8, ValueSize::Dword).u32_exact()?;
        let size = vm.get_reg(Reg32::EDX as u8, ValueSize::Dword).u32_exact()?;
        let data = vm.copy_from_memory(address, size).unwrap();
        self.sccs.push(data.to_vec());
        vm.gas_remaining; //+= data.len() * memory_copy_cost + sccs_push_cost;
        Ok(())
    }
    fn pop_sccs(&mut self, vm: &mut VM) -> Result<(), NeutronError>{
        //let p = self.sccs.pop().ok_or(NeutronError::RecoverableFailure)?;
        let address = vm.get_reg(Reg32::EAX as u8, ValueSize::Dword).u32_exact()?;
        let max_size = vm.get_reg(Reg32::EDX as u8, ValueSize::Dword);
        let data = self.sccs.pop().unwrap(); //returns slice
        vm.copy_into_memory(address, &data);
        vm.gas_remaining; //+= max(max_size, data.len()); /* * sccs_copy_cost )*/
        Ok(())
    }
    fn pop_sccs_toss(&mut self, vm: &mut VM) -> Result<(), NeutronError>{
        if self.sccs.len() == 0{
            Err(NeutronError::RecoverableFailure)
        }else{
            let _ = self.sccs.remove(self.sccs.len() - 1);
            Ok(())
        }
    }
    fn peek_sccs(&mut self, vm: &mut VM) -> Result<(), NeutronError>{
        let address = vm.get_reg(Reg32::EAX as u8, ValueSize::Dword).u32_exact()?;
        let max_size = vm.get_reg(Reg32::ECX as u8, ValueSize::Dword).u32_exact()?;
        let index = vm.get_reg(Reg32::EDX as u8, ValueSize::Dword).u32_exact()? as usize;
        if self.sccs.len() == 0{
            Err(NeutronError::RecoverableFailure)
        } else if index + 1 > self.sccs.len() {
            Err(NeutronError::UnrecoverableFailure)
        } else {
            let data = &self.sccs[self.sccs.len() - (index + 1)];
            vm.copy_into_memory(address, data);
            vm.gas_remaining; // if eax == 0 then 0 else mem_copy_cost * data.len()
            Ok(())
        }
    }
    fn sccs_swap(&mut self, vm: &mut VM) -> Result<(), NeutronError> {
        let index = vm.get_reg(Reg32::EAX as u8, ValueSize::Dword).u32_exact()? as usize;
        if index < 1 {
            Err(NeutronError::UnrecoverableFailure)
        } else {
            let to_switch_index = self.sccs.len() - (index + 1);
            let to_switch_val = self.sccs[to_switch_index];
            let top_val = self.sccs[self.sccs.len() - 1];
            self.sccs[to_switch_index] = top_val;
            self.sccs[self.sccs.len() - 1] = to_switch_val;
            vm.gas_remaining; // -= mem_copy_cost * (top_val.len() + to_switch_val.len())
            Ok(())
        }
    }
    fn sccs_dup(&mut self, vm: &mut VM) -> Result<(), NeutronError> {
        let index = vm.get_reg(Reg32::EAX as u8, ValueSize::Dword).u32_exact()? as usize;
        if index < 1 {
            Err(NeutronError::UnrecoverableFailure)
        } else {
            let to_dup_val = self.sccs[self.sccs.len() - (index + 1)];
            self.sccs.push(to_dup_val);
            vm.gas_remaining; // -= sccs_push_cost;
            Ok(())
        }
    }

    fn sccs_item_count(self, vm: &mut VM) -> Result<(), NeutronError> {
        let num = self.sccs.len();
        vm.set_reg32(Reg32::EAX, num as u32);
        Ok(())
    }
    /*fn peek_sccs_size(&mut self) -> Result<usize, NeutronError>{
        Ok(self.sccs.len())
	}*/
}

/*pub struct TestHypervisor {
    pub api: Box<TestbenchAPI>
}*/


/*impl Hypervisor for TestHypervisor{
    fn interrupt(&mut self, vm: &mut VM, num: u8) -> Result<(), VMError>{
        use TestbenchSyscalls::*;
        if num == NEUTRON_INTERRUPT || num == EXIT_INTERRUPT{
            return (self).interrupt(vm, num);
        }

        if num != TESTBENCH_INTERRUPT{
            //self.api.log_error("Invalid interrupt triggered");
            return Ok(());
        }
        let syscall:TestbenchSyscalls =  FromPrimitive::from_u32(vm.reg32(Reg32::EAX)).unwrap_or(TestbenchSyscalls::Invalid);
        match syscall{
            LogError => {
                //(char *msg, uint32 msg_size) -> void
                let size = vm.reg32(Reg32::ECX);
                let msg = String::from_utf8_lossy(vm.copy_from_memory(vm.reg32(Reg32::EBX), size)?).to_owned();
                //self.api.log_error(&msg);
            },
            LogInfo => {
                //(char *msg, uint32 msg_size) -> void
                let size = vm.reg32(Reg32::ECX);
                let msg = String::from_utf8_lossy(vm.copy_from_memory(vm.reg32(Reg32::EBX), size)?).to_owned();
                //self.api.log_info(&msg);;
            },
            LogDebug => {
                //(char *msg, uint32 msg_size) -> void
                let size = vm.reg32(Reg32::ECX);
                let msg = String::from_utf8_lossy(vm.copy_from_memory(vm.reg32(Reg32::EBX), size)?).to_owned();
                //self.api.log_debug(&msg);
            },
            Invalid => {
                //self.api.log_error("Invalid testbench system call");
            }
            _ => unimplemented!()
        }

        Ok(())
    }
}
*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    
}