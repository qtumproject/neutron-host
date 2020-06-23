extern crate neutron_host;
fn main() {
    println!("Use `cargo test` to execute test suite");
}
#[cfg(test)]
mod tests {
    use neutron_host::testbench::*;
    use neutron_host::addressing::*;
    use neutron_host::callstack::*;
    #[test]
    fn test_contract_create_and_call() {
        let mut address = NeutronAddress::default();
        address.set_to_random_address();
        address.version = 2;
        let mut testbench = Testbench::default();
        //test deploying a new contract from ELF file
        println!("wtf");
        let mut stack = ContractCallStack::default();
        stack.create_top_level_deploy(address.clone(), NeutronAddress::new_random_address(), 10000000, 0);
        println!("wtf");
        let result = testbench.deploy_from_elf(&mut stack, "../../neutron-test/target/i486-neutron/debug/neutron-test".to_string()).unwrap();
        println!("wtf");
        assert!(result.error_code == 0); //ensure no error from contract code
        //test that calling deployed contract works
        let mut stack = ContractCallStack::default();
        //stack.push_sccs(&[10, 20, 30, 40]).unwrap();
        stack.create_top_level_call(address.clone(), NeutronAddress::new_random_address(), 100000, 0);
        let result = testbench.execute_top_context(&mut stack).unwrap();
        //assert!(result.error_code == 0); //ensure no error from contract code
        //test that calling deployed contract works
        let mut stack = ContractCallStack::default();
        stack.create_top_level_call(address.clone(), NeutronAddress::new_random_address(), 100000, 0);
        let result = testbench.execute_top_context(&mut stack).unwrap();
        assert!(result.error_code == 0); //ensure no error from contract code
        // ensure doesn’t work with a new random address
        let mut stack = ContractCallStack::default();
        address.set_to_random_address();
        stack.create_top_level_call(address, NeutronAddress::new_random_address(), 100000, 0);
        assert!(testbench.execute_top_context(&mut stack).is_err())
    }
}