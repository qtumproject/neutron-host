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
    fn execute(&mut self) -> Result<NeutronVMResult, NeutronError>{
        let ctx = self.call_stack.current_context();
        match ctx.execution_type{
            ExecutionType::BareExecution => {
                //..
            },
            ExecutionType::Call => {
                return self.call();
            },
            ExecutionType::Deploy => {
                return self.deploy();
            }
        }
        Err(NeutronError::RecoverableFailure) //todo
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

    fn deploy(&mut self) -> Result<NeutronVMResult, NeutronError>{
        let mut vm = VM::default();
        println!("starting x86");
        if self.init_cpu(&mut vm).is_err(){
            return Err(NeutronError::UnrecoverableFailure);
        }
        println!("x86 initialized");
        self.create_contract_from_sccs(&mut vm)?;
        let result = vm.execute(self);
        if result.is_err(){
            return Err(NeutronError::UnrecoverableFailure);
        }else{
            //???
        }
        vm.print_diagnostics();
        let return_code = vm.reg32(Reg32::EAX);
        if return_code != 0 {
            //if contract signaled error (but didn't actually crash/fail) then exit
            return Err(NeutronError::RecoverableFailure);
        }
        self.store_contract_code()?;
        let r = NeutronVMResult{
            gas_used: self.call_stack.current_context().gas_limit.saturating_sub(vm.gas_remaining),
            should_revert: false,
            error_code: return_code,
            error_location: 0,
            extra_data: 0
        };
        Ok(r)
    }

    fn call(&mut self) -> Result<NeutronVMResult, NeutronError>{
        let mut vm = VM::default();
        println!("starting x86 call");
        if self.init_cpu(&mut vm).is_err(){
            return Err(NeutronError::UnrecoverableFailure);
        }
        println!("x86 initialized");
        self.call_contract_from_sccs(&mut vm)?;
        let result = vm.execute(self);
        if result.is_err(){
            println!("VM error");
            return Err(NeutronError::UnrecoverableFailure);
        }else{
            //???
        }
        vm.print_diagnostics();
        let return_code = vm.reg32(Reg32::EAX);
        if return_code != 0 {
            //if contract signaled error (but didn't actually crash/fail) then exit
            return Err(NeutronError::RecoverableFailure);
        }
        let r = NeutronVMResult{
            gas_used: self.call_stack.current_context().gas_limit.saturating_sub(vm.gas_remaining),
            should_revert: false,
            error_code: return_code,
            error_location: 0,
            extra_data: 0
        };
        Ok(r)
    }

    fn store_contract_code(&mut self) -> Result<(), NeutronError>{
        let code_key = vec![X86Interface::CODE_SECTION_SPACE, 0];
        let data_key = vec![X86Interface::DATA_SECTION_SPACE, 0];
        self.call_system.write_state_key(self.call_stack, X86Interface::X86_SPACE, &code_key, &self.code_sections[0])?;
        self.call_system.write_state_key(self.call_stack, X86Interface::X86_SPACE, &data_key, &self.data_sections[0])?;
        Ok(())
    }
    fn load_contract_code(&mut self) -> Result<(), NeutronError>{
        //todo need to store section counts
        let code_key = vec![X86Interface::CODE_SECTION_SPACE, 0];
        let data_key = vec![X86Interface::DATA_SECTION_SPACE, 0];
        self.code_sections.push(self.call_system.read_state_key(self.call_stack, X86Interface::X86_SPACE, &code_key)?);
        self.data_sections.push(self.call_system.read_state_key(self.call_stack, X86Interface::X86_SPACE, &data_key)?);
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

    fn call_contract_from_sccs(&mut self, vm: &mut VM) -> Result<(), NeutronError>{
        //validate version later on..
        self.load_contract_code()?;

        vm.copy_into_memory(0x10000, &self.code_sections[0]).unwrap();
        vm.copy_into_memory(0x80020000, &self.data_sections[0]).unwrap();

        Ok(())
    }
}

/*
Summary of interface:

Note: returning u64 values uses the EAX:EDX "mostly but not quite" standard cdcel convention

Interrupt 0x10: push_sccs (buffer, size)
Interrupt 0x11: pop_sccs (buffer, max_size) -> actual_size: u32
Interrupt 0x12: peek_sccs (buffer, max_size, index) -> actual_size: u32
Interrupt 0x13: swap_sccs (index)
Interrupt 0x14: dup_sccs()
Interrupt 0x15: gas_remaining()
Interrupt 0x16: exit_execution(status)
Interrupt 0x17: revert_execution(status)
Interrupt 0x18: execution_status()
Interrupt 0x19: sccs_item_count() -> size
Interrupt 0x1A: sccs_memory_size() -> size
Interrupt 0x1B: sccs_memory_remaining() -> size
Interrupt 0x1C: sccs_item_limit_remaining() -> size
Interrupt 0x20: system_call() -> error
-- Hypervisor functions
Interrupt 0x80: alloc_memory TBD
-- Context functions
Interrupt 0x90: gas_used() -> u64
Interrupt 0x91: self_address() -- result on stack as NeutronShortAddress
Interrupt 0x92: origin() -- result on stack as NeutronShortAddress
Interrupt 0x93: origin_long() -- result on stack as NeutronLongAddress
Interrupt 0x94: sender() -- result on stack as NeutronShortAddress
Interrupt 0x95: sender_long() -- result on stack as NeutronLongAddress
Interrupt 0x96: value_sent() -> u64
Interrupt 0x97: nest_level() -> u32
*/

const PUSH_INTERRUPT:u8 = 0x10;
const POP_INTERRUPT:u8 = 0x11;
const PEEK_INTERRUPT:u8 = 0x12;
const SWAP_INTERRUPT:u8 = 0x13;
const DUP_INTERRUPT:u8 = 0x14;
const SYSTEM_CALL_INTERRUPT:u8 = 0x20;
const EXIT_INTERRUPT:u8 = 0xFF;
const REVERT_INTERRUPT:u8 = 0xFE;

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
        if num != 0{
            self.call_system.log_error("Invalid interrupt triggered");
            return Ok(());
        }
        Ok(())
    }
}

