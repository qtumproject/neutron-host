extern crate neutron_star_constants;
extern crate ring;
extern crate struct_deser;
extern crate elf;
use crate::hypervisor::*;
use crate::db::*;
use std::path::PathBuf;
use crate::syscall_interfaces::storage;
use crate::interface::*;
use crate::callstack::*;
use crate::neutronerror::*;
use crate::neutronerror::NeutronError::*;
use crate::syscall_interfaces::logging;

/// The Testbench is a virtual environment which can be used for testing smart contracts 
#[derive(Default)]
pub struct Testbench{
    pub transaction: TransactionContext,
    pub db: ProtoDB
    //etc...
}

impl storage::GlobalStorage for Testbench{
    fn store_state(&mut self, stack: &mut ContractCallStack) -> Result<(), NeutronError>{
        let key = stack.pop_sccs()?;
        let value = stack.pop_sccs()?;
        self.write_state_key(stack, NEUTRONDB_USER_SPACE, &key, &value)
    }
    fn load_state(&mut self, stack: &mut ContractCallStack) -> Result<(), NeutronError>{
        let key = stack.pop_sccs()?;
        let value = self.read_state_key(stack, NEUTRONDB_USER_SPACE, &key)?;
        stack.push_sccs(&value)?;
        Ok(())
    }
    fn key_exists(&mut self, _stack: &mut ContractCallStack) -> Result<(), NeutronError>{
        Err(Unrecoverable(UnrecoverableError::NotImplemented))
    }
}
impl logging::LoggingInterface for Testbench{
    fn log_debug(&mut self, stack: &mut ContractCallStack) -> Result<(), NeutronError>{
        let msg = stack.pop_sccs()?;
        let string = std::string::String::from_utf8_lossy(&msg);
        (self as &mut dyn CallSystem).log_debug(&string);
        Ok(())
    }
    fn log_info(&mut self, stack: &mut ContractCallStack) -> Result<(), NeutronError>{
        let msg = stack.pop_sccs()?;
        let string = std::string::String::from_utf8_lossy(&msg);
        (self as &mut dyn CallSystem).log_info(&string);
        Ok(())
    }
    fn log_warning(&mut self, stack: &mut ContractCallStack) -> Result<(), NeutronError>{
        let msg = stack.pop_sccs()?;
        let string = std::string::String::from_utf8_lossy(&msg);
        (self as &mut dyn CallSystem).log_warning(&string);
        Ok(())
    }
    fn log_error(&mut self, stack: &mut ContractCallStack) -> Result<(), NeutronError>{
        let msg = stack.pop_sccs()?;
        let string = std::string::String::from_utf8_lossy(&msg);
        (self as &mut dyn CallSystem).log_error(&string);
        Ok(())
    }
}

impl CallSystem for Testbench{
    fn system_call(&mut self, stack: &mut ContractCallStack, feature: u32, function: u32) -> Result<u32, NeutronError>{
        //go through each interface implementations until one returns true or an error occurs
        if (self as &mut dyn storage::GlobalStorage).try_syscall(stack, feature, function)? == true{
            return Ok(0);
        }
        if (self as &mut dyn logging::LoggingInterface).try_syscall(stack, feature, function)? == true{
            return Ok(0);
        }


        Ok(0)
    }
    /// Get the current block height at execution
    /// Used to switch VM behavior in blockchain forks
    fn block_height(&self) -> Result<u32, NeutronError>{
        Ok(1)
    }
    /// Read a state key from the database using the permanent storage feature set
    /// Used for reading core contract bytecode by VMs
    fn read_state_key(&mut self, stack: &mut ContractCallStack, space: u8, key: &[u8]) -> Result<Vec<u8>, NeutronError>{
        let mut k = vec![space];
        k.extend_from_slice(key);
        match self.db.read_key(&stack.current_context().self_address.to_short_address(), &k) {
            Err(_e) => {
                Err(Unrecoverable(UnrecoverableError::StateOutOfRent))
            },
            Ok(v) => {
                Ok(v)
            }
        }
    }
    /// Write a state key to the database using the permanent storage feature set
    /// Used for writing bytecode etc by VMs
    fn write_state_key(&mut self, stack: &mut ContractCallStack, space: u8, key: &[u8], value: &[u8]) -> Result<(), NeutronError>{
        let mut k = vec![space];
        k.extend_from_slice(key);
        if self.db.write_key(&stack.current_context().self_address.to_short_address(), &k, value).is_err(){
            Err(Unrecoverable(UnrecoverableError::DatabaseWritingError))
        }else{
            Ok(())
        }
    }
}

impl Testbench{
    /// Begins execution using the top context within the stack
    pub fn execute_top_context(&mut self, stack: &mut ContractCallStack) -> Result<NeutronVMResult, NeutronError>{
        self.db.checkpoint().unwrap();
        if stack.current_context().self_address.version == 2 {
            let mut vm = X86Interface::new(self, stack);
            println!("Executing x86 VM");
            match vm.execute(){
                Err(e) => {
                    self.db.clear_checkpoints();
                    return Err(e);
                },
                Ok(v) => {
                    if self.db.commit().is_err(){
                        println!("database error with commit");
                        self.db.clear_checkpoints();
                        return Err(Unrecoverable(UnrecoverableError::DatabaseCommitError));
                    }
                    return Ok(v);
                }
            }
        }else{
            return Err(Unrecoverable(UnrecoverableError::UnknownVM));
        }
    }
    
    /// Deploy a smart contract from an ELF executable file
    pub fn deploy_from_elf(&mut self, stack: &mut ContractCallStack, file: String) -> Result<NeutronVMResult, NeutronError>{
        assert!(stack.context_count()? == 1, "Exactly one context should be pushed to the ContractCallStack");
        let path = PathBuf::from(file);
        let file = elf::File::open_path(&path).unwrap();
    
        let text_scn = file.get_section(".text").unwrap();
        assert!(text_scn.shdr.addr == 0x10000);
        let data_scn = file.get_section(".data").unwrap();
        assert!(data_scn.shdr.addr == 0x80020000);
    
        stack.push_sccs(&data_scn.data).unwrap();
        stack.push_sccs(&text_scn.data).unwrap();
        let section_info = vec![1, 1];
        stack.push_sccs(&section_info).unwrap(); //code section count
        stack.push_sccs(&vec![2, 0, 0, 0]).unwrap(); //vmversion (fill in properly later)

        self.execute_top_context(stack)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}