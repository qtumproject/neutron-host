extern crate qx86;
extern crate neutron_star_constants;

use qx86::vm::*;
use crate::*;
use interface::*;
use crate::callstack::*;
use crate::neutronerror::NeutronError::*;
use crate::neutronerror::*;

use std::cmp;

/*
Summary of interface:

Note: returning u64 values uses the EAX:EDX "mostly but not quite" standard cdcel convention
Order of registers: EAX, ECX, EDX

-- SCCS functions
Interrupt 0x10: push_sccs (buffer, size)
Interrupt 0x11: pop_sccs (buffer, max_size) -> actual_size: u32
Interrupt 0x12: peek_sccs (buffer, max_size, index) -> actual_size: u32
Interrupt 0x13: swap_sccs (index)
Interrupt 0x14: dup_sccs()
Interrupt 0x15: sccs_item_count() -> size
Interrupt 0x16: sccs_memory_size() -> size
Interrupt 0x17: sccs_memory_remaining() -> size
Interrupt 0x18: sccs_item_limit_remaining() -> size

-- CallSystem functions
Interrupt 0x20: system_call(feature, function) -> error:u32

-- Hypervisor functions
Interrupt 0x80: alloc_memory TBD

-- Context functions
Interrupt 0x90: gas_limit() -> u64
Interrupt 0x91: self_address() -- result on stack as NeutronShortAddress
Interrupt 0x92: origin() -- result on stack as NeutronShortAddress
Interrupt 0x93: origin_long() -- result on stack as NeutronLongAddress
Interrupt 0x94: sender() -- result on stack as NeutronShortAddress
Interrupt 0x95: sender_long() -- result on stack as NeutronLongAddress
Interrupt 0x96: value_sent() -> u64
Interrupt 0x97: nest_level() -> u32
Interrupt 0x98: gas_remaining() -> u64
Interrupt 0x99: execution_type() -> u32

-- System interrupts
Interrupt 0xFE: revert_execution(status) -> noreturn
Interrupt 0xFF: exit_execution(status) -> noreturn

*/

#[derive(FromPrimitive)]
enum StackInterrupt{
    Push = 0x10,
    Pop,
    Peek,
    Swap,
    Dup,
    ItemCount,
    StackSize,
    StackSizeRemaining,
    ItemLimitRemaining

}
#[derive(FromPrimitive)]
enum CallSystemInterrupt{
    SystemCall = 0x20
}
#[derive(FromPrimitive)]
enum HypervisorInterrupt{
    AllocMemory = 0x80
}
#[derive(FromPrimitive)]
enum ExecInfoInterrupt{
    GasLimit = 0x90,
    SelfAddress,
    Origin,
    OriginLong,
    Sender,
    SenderLong,
    ValueSent,
    NestLevel,
    GasRemaining,
    ExecutionType
}
#[derive(FromPrimitive)]
enum SystemInterrupt{
    RevertExecution = 0xFE,
    ExitExecution = 0xFF
}


/// The VM interface for executing x86 smart contracts in Neutron
pub struct X86Interface<'a>{
    pub call_system: &'a mut dyn CallSystem,
    pub call_stack: &'a mut ContractCallStack,
    code_sections: Vec<Vec<u8>>,
    data_sections: Vec<Vec<u8>>
}

impl<'a> VMInterface for X86Interface<'a>{
    /// Begins execution of x86 smart contract by interpreting the type fo execution needed using the current_context
    fn execute(&mut self) -> Result<NeutronVMResult, NeutronError>{
        let ctx = self.call_stack.current_context();
        match ctx.execution_type{
            ExecutionType::BareExecution => {
                //..
                return Err(Unrecoverable(UnrecoverableError::NotImplemented));
            },
            ExecutionType::Call => {
                return self.call();
            },
            ExecutionType::Deploy => {
                return self.deploy();
            }
        }
    }
}

impl<'a> X86Interface<'a> {
    const X86_SPACE: u8 = 2;
    const CODE_SECTION_SPACE: u8 = 1;
    const DATA_SECTION_SPACE: u8 = 2;

    /// Creates a new instance of the X86Interface
    pub fn new<'b>(cs: &'b mut dyn CallSystem, stack: &'b mut ContractCallStack) -> X86Interface<'b>{
        X86Interface{
            call_stack: stack,
            call_system: cs,
            code_sections: Vec::default(),
            data_sections: Vec::default()
        }
    }
    
    fn deploy(&mut self) -> Result<NeutronVMResult, NeutronError>{
        let mut vm = VM::default();
        if self.init_cpu(&mut vm).is_err(){
            return Err(Unrecoverable(UnrecoverableError::ErrorInitializingVM));
        }
        self.create_contract_from_sccs(&mut vm)?;
        let result = vm.execute(self);
        if result.is_err(){
            vm.print_diagnostics();
            self.call_system.log_warning(&format!("Contract encountered an execution error: {:?}", result.unwrap_err()));
            vm.print_diagnostics();
            return Err(Recoverable(RecoverableError::ContractExecutionError));
        }
        let return_code = vm.reg32(Reg32::EAX);
        if return_code != 0 {
            //if contract signaled error (but didn't actually crash/fail) then exit
            return Err(Recoverable(RecoverableError::ContractSignaledError));
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
        if self.init_cpu(&mut vm).is_err(){
            return Err(Unrecoverable(UnrecoverableError::ErrorInitializingVM));
        }
        self.call_contract_from_sccs(&mut vm)?;
        let result = vm.execute(self);
        if result.is_err(){
            self.call_system.log_warning(&format!("Contract encountered an execution error: {:?}", result.unwrap_err()));
            vm.print_diagnostics();
            return Err(Recoverable(RecoverableError::ContractExecutionError));
        }
        let return_code = vm.reg32(Reg32::EAX);
        if return_code != 0 {
            //if contract signaled error (but didn't actually crash/fail) then exit
            return Err(Recoverable(RecoverableError::ContractSignaledError));
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

    /// Will store all of the currently loaded code and data sections using the associated CallSystem's storage functions
    fn store_contract_code(&mut self) -> Result<(), NeutronError>{
        let code_key = vec![X86Interface::CODE_SECTION_SPACE, 0];
        let data_key = vec![X86Interface::DATA_SECTION_SPACE, 0];
        self.call_system.write_state_key(self.call_stack, X86Interface::X86_SPACE, &code_key, &self.code_sections[0])?;
        self.call_system.write_state_key(self.call_stack, X86Interface::X86_SPACE, &data_key, &self.data_sections[0])?;
        Ok(())
    }
    /// Will load all of the currently available code and data sections using the associated CallSystem's storage functions
    fn load_contract_code(&mut self) -> Result<(), NeutronError>{
        //todo need to store section counts
        let code_key = vec![X86Interface::CODE_SECTION_SPACE, 0];
        let data_key = vec![X86Interface::DATA_SECTION_SPACE, 0];
        self.code_sections.push(self.call_system.read_state_key(self.call_stack, X86Interface::X86_SPACE, &code_key)?);
        self.data_sections.push(self.call_system.read_state_key(self.call_stack, X86Interface::X86_SPACE, &data_key)?);
        Ok(())
    }

    /// Will create a new instance of an x86 VM
    fn init_cpu(&mut self, vm: &mut VM) -> Result<(), VMError>{
        self.init_memory(vm)?;
        vm.charger = self.call_stack.x86_gas_charger();
        vm.gas_remaining = self.call_stack.current_context().gas_limit;
        vm.eip = 0x10000;
        Ok(())
    }
    /// Initializes all of the memory areas that are expected to be within a Neutron-x86 VM
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
    /// Will create a new contract using data pushed onto the SCCS and current_context
    fn create_contract_from_sccs(&mut self, vm: &mut VM) -> Result<(), NeutronError>{
        //validate version later on..
        let _version = self.call_stack.pop_sccs()?;

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
    /// Will call an existing contract by using data pushed onto the SCCS and current_context
    fn call_contract_from_sccs(&mut self, vm: &mut VM) -> Result<(), NeutronError>{
        //validate version later on..
        self.load_contract_code()?;

        vm.copy_into_memory(0x10000, &self.code_sections[0]).unwrap();
        vm.copy_into_memory(0x80020000, &self.data_sections[0]).unwrap();

        Ok(())
    }
    /// Handles all SCCS interrupts
    fn stack_interrupt(&mut self, vm: &mut VM, function: StackInterrupt) -> Result<(), NeutronError>{
        match function{
            StackInterrupt::Push => {
                let address = vm.reg32(Reg32::EAX);
                let size = vm.reg32(Reg32::ECX);
                self.call_stack.charge_gas(self.call_stack.gas_cost(INTERNAL_BUILT_IN_FEATURE, CallStackCost::CopyDataFromVM as u32) * size as i64)?;
                let memory = vm.copy_from_memory(address, size);
                if memory.is_err(){
                    return Err(Recoverable(RecoverableError::ErrorCopyingFromVM));
                }
                let memory = memory.unwrap();
                self.call_stack.push_sccs(memory)?;
                vm.set_reg32(Reg32::EAX, 0);
            },
            StackInterrupt::Pop => {
                let memory = self.call_stack.pop_sccs()?;
                let address = vm.reg32(Reg32::EAX);
                let max_size = cmp::min(vm.reg32(Reg32::ECX) as usize, memory.len());
                self.call_stack.charge_gas(self.call_stack.gas_cost(INTERNAL_BUILT_IN_FEATURE, CallStackCost::CopyDataToVM as u32) * max_size as i64)?;
                if address != 0 && max_size != 0{
                    let result = vm.copy_into_memory(address, &memory[0..max_size]);
                    if result.is_err(){
                        return Err(Recoverable(RecoverableError::ErrorCopyingIntoVM));
                    }
                }
                vm.set_reg32(Reg32::EAX, memory.len() as u32); //set EAX to actual_size
            },
            StackInterrupt::Peek => {
                let index = vm.reg32(Reg32::EDX);
                let memory = self.call_stack.peek_sccs(index)?;
                let address = vm.reg32(Reg32::EAX);
                let max_size = cmp::min(vm.reg32(Reg32::ECX) as usize, memory.len());
                self.call_stack.charge_gas(self.call_stack.gas_cost(INTERNAL_BUILT_IN_FEATURE, CallStackCost::CopyDataToVM as u32) * max_size as i64)?;
                if address != 0 && max_size != 0{
                    let result = vm.copy_into_memory(address, &memory[0..max_size]);
                    if result.is_err(){
                        return Err(Recoverable(RecoverableError::ErrorCopyingIntoVM));
                    }
                }
                vm.set_reg32(Reg32::EAX, memory.len() as u32); //set EAX to actual_size
            }
            _ => {}
        };
        Ok(())
    }
    /// Will translate NeutronError into appropriate register values in the VM and to trigger a proper unrecoverable VMError if needed
    fn translate_interrupt_result(&mut self, vm: &mut VM, result: Result<(), NeutronError>) -> Result<(), VMError>{
        match result{
            Ok(_) => {
                self.sync_gas(vm);
                return Ok(());
            },
            Err(e) => {
                match e{
                    Unrecoverable(x) => {
                        self.call_system.log_warning(&format!("Unrecoverable hypervisor error: {:?}", x));
                        self.sync_gas(vm);
                        return Err(VMError::SyscallError);
                    },
                    Recoverable(x) => {
                        if x == RecoverableError::OutOfGas{
                            vm.gas_remaining = 0;
                            return Err(VMError::OutOfGas);
                        }
                        self.call_system.log_debug(&format!("Recoverable hypervisor error: {:?}", x));
                        vm.set_reg32(Reg32::EAX, x as u32);
                        self.sync_gas(vm);
                        return Ok(());
                    }
                }
            }
        }
    }
    fn sync_gas(&mut self, vm: &mut VM){
        vm.gas_remaining = (vm.gas_remaining as i64 - self.call_stack.pending_gas) as u64;
        self.call_stack.pending_gas = 0;
        self.call_stack.gas_remaining = 0;
    }
}

impl <'a> Hypervisor for X86Interface<'a> {
    /// The primary interface into the hypervisor from the VM programs. This is triggered by using an `INT` opcode within the VM program
    fn interrupt(&mut self, vm: &mut VM, num: u8) -> Result<(), VMError>{
        self.call_stack.gas_remaining = vm.gas_remaining;
        let call = num::FromPrimitive::from_u8(num);
        if call.is_some(){
            let result = self.stack_interrupt(vm, call.unwrap());
            return self.translate_interrupt_result(vm, result);
        }

        if num == SystemInterrupt::ExitExecution as u8{
            self.call_system.log_debug("Exit interrupt triggered");
            return Err(VMError::InternalVMStop);
        }
        if num == CallSystemInterrupt::SystemCall as u8{
            let feature = vm.reg32(Reg32::EAX);
            let function = vm.reg32(Reg32::ECX);
            let result = self.call_system.system_call(self.call_stack, feature, function);
            //how to handle unrecoverable??
            match result{
                Err(e) => {
                    match e{
                        Unrecoverable(x) => {
                            self.call_system.log_warning(&format!("Unrecoverable system call error: {:?}", x));
                            self.sync_gas(vm);
                            return Err(VMError::SyscallError);
                        },
                        Recoverable(x) => {
                            if x == RecoverableError::OutOfGas{
                                self.call_system.log_debug("Ran out of gas during system call execution");
                                vm.gas_remaining = 0;
                                return Err(VMError::OutOfGas);
                            }
                            self.call_system.log_debug(&format!("Recoverable system call error: {:?}", x));
                            vm.set_reg32(Reg32::EAX, x as u32);
                            self.sync_gas(vm);
                            return Ok(());
                        }
                    }
                },
                Ok(v) => {
                    vm.set_reg32(Reg32::EAX, v);
                    self.sync_gas(vm);
                    return Ok(())
                }
            }
        }
        if num == ExecInfoInterrupt::ExecutionType as u8{
            vm.set_reg32(Reg32::EAX, self.call_stack.current_context().execution_type as u32);
            self.sync_gas(vm);
            return Ok(());
        }
        if num != 0{
            self.call_system.log_warning(&format!("Invalid interrupt triggered: {:?}", num));
            vm.set_reg32(Reg32::EAX, RecoverableError::InvalidHypervisorInterrupt as u32);
            self.sync_gas(vm);
            return Ok(());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct DummyCallSystem{}
    impl CallSystem for DummyCallSystem{
        fn system_call(&mut self, _stack: &mut ContractCallStack, _feature: u32, _function: u32) -> Result<u32, NeutronError>{
            Err(Unrecoverable(UnrecoverableError::NotImplemented))
        }
        fn block_height(&self) -> Result<u32, NeutronError>{
            Err(Unrecoverable(UnrecoverableError::NotImplemented))
        }
        fn read_state_key(&mut self, _stack: &mut ContractCallStack, _space: u8, _key: &[u8]) -> Result<Vec<u8>, NeutronError>{
            Err(Unrecoverable(UnrecoverableError::NotImplemented))
        }
        /// Write a state key to the database using the permanent storage feature set
        /// Used for writing bytecode etc by VMs
        fn write_state_key(&mut self, _stack: &mut ContractCallStack, _space: u8, _key: &[u8], _value: &[u8]) -> Result<(), NeutronError>{
            Err(Unrecoverable(UnrecoverableError::NotImplemented))
        }
    }
    #[test]
    fn test_x86_sccs_push(){
        let mut stack = ContractCallStack::default();
        let mut cs = DummyCallSystem{};
        {
            let mut hv = X86Interface::new(&mut cs, &mut stack);
            let mut vm = qx86::vm::VM::default();
            let address = 0x8000_0000;
            vm.memory.add_memory(0x8000_0000, 0x100).unwrap();
            let item = vec![0, 1, 2, 3, 4];
            vm.copy_into_memory(address, &item).unwrap();
            vm.set_reg32(Reg32::EAX, address);
            vm.set_reg32(Reg32::ECX, 5);
            hv.interrupt(&mut vm, StackInterrupt::Push as u8).unwrap();
            vm.set_reg32(Reg32::EAX, address);
            vm.set_reg32(Reg32::ECX, 2);
            hv.interrupt(&mut vm, StackInterrupt::Push as u8).unwrap();
        }
        assert_eq!(stack.sccs_item_count().unwrap(), 2);
        let item = stack.pop_sccs().unwrap();
        assert_eq!(item, vec![0, 1]);
        let item = stack.pop_sccs().unwrap();
        assert_eq!(item, vec![0, 1, 2, 3, 4]);
        assert_eq!(stack.sccs_item_count().unwrap(), 0);
    }
    #[test]
    fn test_x86_sccs_pop(){
        let mut stack = ContractCallStack::default();
        let mut cs = DummyCallSystem{};
        let item = vec![9, 1, 2, 3, 4];
        stack.push_sccs(&item).unwrap();
        stack.push_sccs(&item).unwrap();
        stack.push_sccs(&item[0..2]).unwrap();
        {
            let mut hv = X86Interface::new(&mut cs, &mut stack);
            let mut vm = qx86::vm::VM::default();
            let address = 0x8000_0000;
            vm.memory.add_memory(0x8000_0000, 0x100).unwrap();
            vm.set_reg32(Reg32::EAX, address);
            vm.set_reg32(Reg32::ECX, 5); //max_size
            hv.interrupt(&mut vm, StackInterrupt::Pop as u8).unwrap();
            assert_eq!(vm.reg32(Reg32::EAX), 2, "VM got incorrect actual_size for SCCS item");
            let data = vm.copy_from_memory(address, 5).unwrap();
            assert_eq!(data.to_vec(), vec![9 as u8, 1, 0, 0, 0], "VM had incorrect data written into memory for SCCS item");

            vm.set_reg32(Reg32::EAX, address + 0x10);
            vm.set_reg32(Reg32::ECX, 2); //max_size
            hv.interrupt(&mut vm, StackInterrupt::Pop as u8).unwrap();
            assert_eq!(vm.reg32(Reg32::EAX), 5, "VM got incorrect actual_size for SCCS item");
            let data = vm.copy_from_memory(address + 0x10, 5).unwrap();
            assert_eq!(data.to_vec(), vec![9 as u8, 1, 0, 0, 0], "VM had incorrect data written into memory for SCCS item");

            vm.set_reg32(Reg32::EAX, address + 0x20);
            vm.set_reg32(Reg32::ECX, 5); //max_size
            hv.interrupt(&mut vm, StackInterrupt::Pop as u8).unwrap();
            assert_eq!(vm.reg32(Reg32::EAX), 5, "VM got incorrect actual_size for SCCS item");
            let data = vm.copy_from_memory(address + 0x20, 5).unwrap();
            assert_eq!(data.to_vec(), vec![9 as u8, 1, 2, 3, 4], "VM had incorrect data written into memory for SCCS item");
        }
        assert_eq!(stack.sccs_item_count().unwrap(), 0);
    }
    #[test]
    fn test_x86_sccs_drop(){
        let mut stack = ContractCallStack::default();
        let mut cs = DummyCallSystem{};
        let item = vec![9, 1, 2, 3, 4];
        stack.push_sccs(&item).unwrap();
        {
            let mut hv = X86Interface::new(&mut cs, &mut stack);
            let mut vm = qx86::vm::VM::default();
            let address = 0; //null pointer
            vm.set_reg32(Reg32::EAX, address);
            vm.set_reg32(Reg32::ECX, 0); //max_size (null)
            hv.interrupt(&mut vm, StackInterrupt::Pop as u8).unwrap();
            assert_eq!(vm.reg32(Reg32::EAX), 5, "VM got incorrect actual_size for SCCS item");
        }
        assert_eq!(stack.sccs_item_count().unwrap(), 0);
    }
    #[test]
    fn test_x86_sccs_peek(){
        let mut stack = ContractCallStack::default();
        let mut cs = DummyCallSystem{};
        let item = vec![9, 1, 2, 3, 4];
        stack.push_sccs(&item).unwrap();
        stack.push_sccs(&item).unwrap();
        stack.push_sccs(&item[0..2]).unwrap();
        {
            let mut hv = X86Interface::new(&mut cs, &mut stack);
            let mut vm = qx86::vm::VM::default();
            let address = 0x8000_0000;
            vm.memory.add_memory(0x8000_0000, 0x100).unwrap();
            vm.set_reg32(Reg32::EAX, address);
            vm.set_reg32(Reg32::ECX, 5); //max_size
            vm.set_reg32(Reg32::EDX, 0); //index
            hv.interrupt(&mut vm, StackInterrupt::Peek as u8).unwrap();
            assert_eq!(vm.reg32(Reg32::EAX), 2, "VM got incorrect actual_size for SCCS item");
            let data = vm.copy_from_memory(address, 5).unwrap();
            assert_eq!(data.to_vec(), vec![9 as u8, 1, 0, 0, 0], "VM had incorrect data written into memory for SCCS item");

            vm.set_reg32(Reg32::EAX, address + 0x10);
            vm.set_reg32(Reg32::ECX, 2); //max_size
            vm.set_reg32(Reg32::EDX, 1); //index
            hv.interrupt(&mut vm, StackInterrupt::Peek as u8).unwrap();
            assert_eq!(vm.reg32(Reg32::EAX), 5, "VM got incorrect actual_size for SCCS item");
            let data = vm.copy_from_memory(address + 0x10, 5).unwrap();
            assert_eq!(data.to_vec(), vec![9 as u8, 1, 0, 0, 0], "VM had incorrect data written into memory for SCCS item");

            vm.set_reg32(Reg32::EAX, address + 0x20);
            vm.set_reg32(Reg32::ECX, 5); //max_size
            vm.set_reg32(Reg32::EDX, 2); //index
            hv.interrupt(&mut vm, StackInterrupt::Peek as u8).unwrap();
            assert_eq!(vm.reg32(Reg32::EAX), 5, "VM got incorrect actual_size for SCCS item");
            let data = vm.copy_from_memory(address + 0x20, 5).unwrap();
            assert_eq!(data.to_vec(), vec![9 as u8, 1, 2, 3, 4], "VM had incorrect data written into memory for SCCS item");

            //null buffer test
            vm.set_reg32(Reg32::EAX, 0);
            vm.set_reg32(Reg32::ECX, 0); //max_size
            vm.set_reg32(Reg32::EDX, 2); //index
            hv.interrupt(&mut vm, StackInterrupt::Peek as u8).unwrap();
            assert_eq!(vm.reg32(Reg32::EAX), 5, "VM got incorrect actual_size for SCCS item");
        }
        assert_eq!(stack.sccs_item_count().unwrap(), 3);
    }
}

