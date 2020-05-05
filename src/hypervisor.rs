extern crate qx86;
extern crate neutron_star_constants;
use qx86::vm::*;
use crate::*;
use neutron_star_constants::*;
use interface::*;




pub struct X86Interface<'a>{
    pub call_system: &'a mut dyn CallSystem,
    pub call_stack: &'a mut ContractCallStack,
    code_sections: Vec<Vec<u8>>,
    data_sections: Vec<Vec<u8>>
}

impl<'a> VMInterface for X86Interface<'a>{
    fn execute(&mut self) -> Result<u32, NeutronError>{
        let ctx = self.call_stack.current_context();
        match ctx.execution_type{
            ExecutionType::BareExecution => {
                //..
            },
            ExecutionType::Call => {

            },
            ExecutionType::Deploy => {
                return self.deploy();
            }
        }
        Ok(0)
    }
}

impl<'a> X86Interface<'a> {
    const X86_SPACE: u8 = 2;
    const CODE_SECTION_SPACE: u8 = 1;
    const DATA_SECTION_SPACE: u8 = 2;

    pub fn new<'b>(cs: &'b mut CallSystem, stack: &'b mut ContractCallStack) -> X86Interface<'b>{
        X86Interface{
            call_stack: stack,
            call_system: cs,
            code_sections: Vec::default(),
            data_sections: Vec::default()
        }
    }

    fn deploy(&mut self) -> Result<u32, NeutronError>{
        let mut vm = VM::default();
        if self.init_cpu(&mut vm).is_err(){
            return Err(NeutronError::UnrecoverableFailure);
        }
        self.create_contract_from_sccs(&mut vm)?;
        let result = vm.execute(self);
        if result.is_err(){
            return Err(NeutronError::UnrecoverableFailure);
        }else{
            //???
        }
        if vm.reg32(Reg32::EAX) != 0 {
            //if contract signaled error (but didn't actually crash/fail) then exit
            return Err(NeutronError::RecoverableFailure);
        }
        self.store_contract_code()?;
        Ok(0)
    }

    fn store_contract_code(&mut self) -> Result<(), NeutronError>{
        let code_key = vec![X86Interface::CODE_SECTION_SPACE, 0];
        let data_key = vec![X86Interface::DATA_SECTION_SPACE, 0];
        self.call_system.write_state_key(X86Interface::X86_SPACE, &code_key, &self.code_sections[0])?;
        self.call_system.write_state_key(X86Interface::X86_SPACE, &data_key, &self.data_sections[0])?;
        Ok(())
    }

    pub fn init_cpu(&mut self, vm: &mut VM) -> Result<(), VMError>{
        self.init_memory(vm)?;
        vm.gas_remaining = self.call_stack.current_context().gas_limit;
        vm.eip = 0x10000;
        Ok(())
    }
    fn init_memory(&mut self, vm: &mut VM) -> Result<(), VMError>{
        //for now, just make all memories max size
        //code memories
        vm.memory.add_memory(0x10000, 0xFFFF)?;
        /* later when we support multiple sections
        vm.memory.add_memory(0x20000, 0xFFFF)?;
        vm.memory.add_memory(0x30000, 0xFFFF)?;
        vm.memory.add_memory(0x40000, 0xFFFF)?;
        vm.memory.add_memory(0x50000, 0xFFFF)?;
        vm.memory.add_memory(0x60000, 0xFFFF)?;
        vm.memory.add_memory(0x70000, 0xFFFF)?;
        */

        //should exec/tx/blockchain data still be exposed somehow directly in memory?
        //exec data
        vm.memory.add_memory(0x70000000, 0xFFFF)?;
        //tx data
        vm.memory.add_memory(0x70010000, 0xFFFF)?;
        //blockchain data
        vm.memory.add_memory(0x70020000, 0xFFFF)?;

        //RAM
        //stack memory
        vm.memory.add_memory(0x80010000, 1024 * 8)?;
        //primary memory
        vm.memory.add_memory(0x80020000, 0xFFFF)?;
        //aux memory
        vm.memory.add_memory(0x80030000, 0xFFFF)?;
        Ok(())
    }

    pub fn create_contract_from_sccs(&mut self, vm: &mut VM) -> Result<(), NeutronError>{
        //validate version later on..
        let version = self.call_stack.pop_sccs()?;

        let section_info = self.call_stack.pop_sccs()?;

        let code_sections = section_info[0];
        assert!(code_sections == 1);
        self.code_sections.push(self.call_stack.pop_sccs()?);

        vm.copy_into_memory(0x10000, &self.code_sections[0]).unwrap();

        let data_sections = section_info[1];
        assert!(data_sections == 1);
        self.data_sections.push(self.call_stack.pop_sccs()?);
        vm.copy_into_memory(0x80020000, &self.data_sections[0]).unwrap();

        //self.call_stack.drop_sccs(); //throw away extra

        Ok(())
    }

    fn call_contract_from_sccs(&mut self, _vm: &mut VM){

    }
}

//todo, move these into neutron-star-constants
const SYSTEM_CALL_INTERRUPT:u8 = 0x20;

impl <'a> Hypervisor for X86Interface<'a> {
    
    fn interrupt(&mut self, vm: &mut VM, num: u8) -> Result<(), VMError>{
        if num == EXIT_INTERRUPT{
            self.call_system.log_debug("Exit interrupt triggered");
            return Err(VMError::InternalVMStop);
        }
        if num == SYSTEM_CALL_INTERRUPT{
            let feature = vm.reg32(Reg32::EAX);
            let function = vm.reg32(Reg32::ECX);
            let result = self.call_system.system_call(self.call_stack, feature, function);
            //how to handle unrecoverable??
            match result{
                Err(e) => {
                    if e == NeutronError::UnrecoverableFailure{
                        return Err(VMError::SyscallError);
                    }else{
                        //use generic error
                        vm.set_reg32(Reg32::EAX, 0xFFFF_FFFF);
                        return Ok(());
                    }
                },
                Ok(v) => {
                    vm.set_reg32(Reg32::EAX, v);
                    return Ok(())
                }
            }
        }
        if num != NEUTRON_INTERRUPT{
            self.call_system.log_error("Invalid interrupt triggered");
            return Ok(());
        }
        Ok(())
    }
}

