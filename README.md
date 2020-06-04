# Neutron Host

This repository holds the various pieces of infrastructure for running and executing Neutron smart contracts. The APIs for actually writing Neutron smart contracts is located at [neutron-star](https://github.com/qtumproject/neutron-star) and [neutron-star-rt](https://github.com/qtumproject/neutron-star-rt). 

Currently most functionality here is for running integration tests on Neutron smart contracts. The primary 2 structures to be concerned with is the following:

## ContractCallStack

This class holds the "context" of execution, as well as the "smart contract communication stack" or 'SCCS'. The context holds information about the current execution (as well as sub-call executions) such as how many coins were sent with the execution, the gas limit, address of the contract being executed, type of execution, etc. The SCCS is a general purpose data stack that is shared between different smart contract executions and the hypervisor interface. It is ideal for ABI data such as specifying what function to call within a smart contract and what arguments to pass to those functions. Internally system calls, such as storing state or creating transactions, also use the SCCS for transferring data to/from the smart contract code to/from the "CallSystem" interface. Later an official NeutronABI system will be implemented so that the SCCS is a purely internal detail that most people never need to actually interact with. 

## Testbench

The Testbench structure implements the "CallSystem" concept of Neutron. The CallSystem is the method by which smart contracts talk to the Testbench, but in real implementations would talk to the underlying blockchain. In addition to this internal use, the Testbench structure also includes (or at least will in the future) all of the things that a smart contract would normally talk to or get information from. This includes concepts like changing balances of an address, calling other smart contracts, getting the current block information, etc. Testbench also includes a connection to "ProtoDB" a very simplistic in-memory database with no cryptographic proofs nor rent implemented for right now. ProtoDB allows for multiple contracts to be deployed into a "fake blockchain" like system and and those smart contracts called over the life of the Testbench. Testbench is designed so that smart contract developers can very specifically create a certain environment across a number of different smart contract calls in order to test that their smart contract functions as expected with programmatic assertions etc. This does not fully replace the need for full in-blockchain testing, but provides an easy to use way to test for specific edge cases (even those that might be impossible within a real blockchain!) that can otherwise be very difficult or cumbersome to consistently reconstruct in a testnet or regtest blockchain environment. 

Until Qtum implements Neutron into a testnet, the Testbench is the best way to actually try out the smart contract capabilities that Neutron implements. However, even after Qtum has a testnet, the Testbench is intended to be a very useful smart contract developer tool that is much easier to use and debug than the traditional regtest blockchain testing strategy. 


## Example

An example usage of the Neutron Host Testbench is below:

```
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
        address.set_to_random_address(); //create a new random address to deploy our contract to
        address.version = 2; //2 means treat as an x86 smart contract. This will later have constants or might change to an enum
        let mut testbench = Testbench::default();
        
        //test deploying contract from a compiled ELF file (ie, what the Rust compiler outputs)
        let mut stack = ContractCallStack::default();
        stack.create_top_level_deploy(address.clone(), NeutronAddress::new_random_address(), 10000000, 0); //create a "context" indicating that this is a smart contract deployment
        let result = testbench.deploy_from_elf(&mut stack, "../my_smart_contract/i486-neutron/debug/my_smart_contract".to_string()).unwrap();
        assert!(result.error_code == 0); //ensure no error returned from our smart contract code

        //test that calling deployed contract works (with no arguments nor function selector ABI passed)
        let mut stack = ContractCallStack::default();
        stack.create_top_level_call(address.clone(), NeutronAddress::new_random_address(), 100000, 0); //create a "context" indicating that this is a smart contract call
        let result = testbench.execute_top_context(&mut stack).unwrap();
        assert!(result.error_code == 0); //ensure no error returned from our smart contract code
        
    }
}
```

Note that the interface for this is still under heavy flux and as more functionality is built, the above example is planned to be greatly simplified to reduce the amount of "internal" knowledge needed for outside users.