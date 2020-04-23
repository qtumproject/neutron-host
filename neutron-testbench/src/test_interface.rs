extern crate neutron_star_constants;
use neutron_host::hypervisor::*;
use neutron_host::interface::*;
use neutron_host::db::*;
use neutron_host::addressing::*;
use qx86::vm::*;
use neutron_star_constants::*;  
use num_traits::FromPrimitive;
use crate::blockchain::SimulatedBlockchain;



#[derive(Clone, Default)]
pub struct TestbenchAPI{
    sccs: Vec<Vec<u8>>,
	pub context: NeutronContext,
	pub chain: SimulatedBlockchain,
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

	fn load_external_state(&mut self, address: &NeutronShortAddress, key: &[u8], data: &mut Vec<u8>) -> Result<usize, NeutronError> {
		Ok(0)
	}

	fn load_external_protected_state(&mut self, address: &NeutronShortAddress, key: &[u8], data: &mut Vec<u8>) -> Result<usize, NeutronError> {
		Ok(0)
	}

    /// Transfers coins from the currently executing smart contract to the specified address
    fn transfer(&mut self, address: &NeutronAddress, value: u64) -> Result<(), NeutronError> {
		Ok(())
	}
    /// Transfers coins from the currently executing smart contract to the specified address
    /// This can only be used for valid short addresses where the amount of data in a full address exactly matches the size of a short address
    fn transfer_short(&mut self, address: &NeutronShortAddress, value: u64) -> Result<(), NeutronError> {
		Ok(())
	}
    /// Returns the balance of the currently executing smart contract
    fn balance(&mut self) -> Result<u64, NeutronError> {
		Ok(0)
	}
    /// Checks the balance of an external smart contract. This can not be used for checking the balance of non-contract addresses.
    fn balance_of_external(&mut self, address: &NeutronShortAddress) -> Result<u64, NeutronError> {
		Ok(0)
	}

    /// Gets the block hash of the specified block
    fn get_block_hash(&mut self, number: u64, hash: &mut[u8]) -> Result<(), NeutronError> {
		Ok(())
	}

    /// Calculates the difference in gas cost produced by changing the amount of allocated memory.
    /// Note this does not actually allocate any memory, this is left to the specific VM and hypervisor.
    /// This is only for charging an appropriate gas cost to the smart contract for allocating/freeing memory.
    fn calculate_memory_cost(&self, existing_size: u64, new_size: u64) -> Result<i64, NeutronError> {
		Ok(0)
	}
    /// Calculates the difference in gas cost produced by changing the amount of allocated read-only memory.
    /// Note this does not actually allocate any memory nor charge the smart contract for the gas, this is left to the specific VM and hypervisor.
    /// This is only for charging an appropriate gas cost to the smart contract for allocating/freeing memory.
    fn calculate_readonly_memory_cost(&self, existing_size: u64, new_size: u64) -> Result<i64, NeutronError> {
		Ok(0)
	}

	fn add_gas_cost(&mut self, gas_difference: i64) -> Result<u64, NeutronError>{
		Ok(0)
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

pub struct TestHypervisor {
    pub api: Box<TestbenchAPI>
}


impl Hypervisor for TestHypervisor{
    fn interrupt(&mut self, vm: &mut VM, num: u8) -> Result<(), VMError>{
        use TestbenchSyscalls::*;
        if num == NEUTRON_INTERRUPT || num == EXIT_INTERRUPT{
            return (self).interrupt(vm, num);
        }

        if num != TESTBENCH_INTERRUPT{
            self.api.log_error("Invalid interrupt triggered");
            return Ok(());
        }
        let syscall:TestbenchSyscalls =  FromPrimitive::from_u32(vm.reg32(Reg32::EAX)).unwrap_or(TestbenchSyscalls::Invalid);
        match syscall{
            LogError => {
                //(char *msg, uint32 msg_size) -> void
                let size = vm.reg32(Reg32::ECX);
                let msg = String::from_utf8_lossy(vm.copy_from_memory(vm.reg32(Reg32::EBX), size)?).to_owned();
                self.api.log_error(&msg);
            },
            LogInfo => {
                //(char *msg, uint32 msg_size) -> void
                let size = vm.reg32(Reg32::ECX);
                let msg = String::from_utf8_lossy(vm.copy_from_memory(vm.reg32(Reg32::EBX), size)?).to_owned();
                self.api.log_info(&msg);;
            },
            LogDebug => {
                //(char *msg, uint32 msg_size) -> void
                let size = vm.reg32(Reg32::ECX);
                let msg = String::from_utf8_lossy(vm.copy_from_memory(vm.reg32(Reg32::EBX), size)?).to_owned();
                self.api.log_debug(&msg);
            },
            Invalid => {
                self.api.log_error("Invalid testbench system call");
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