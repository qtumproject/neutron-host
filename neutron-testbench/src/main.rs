extern crate elf;
extern crate qx86;

use std::env;
use std::path::PathBuf;
use neutron_host::hypervisor::*;
use neutron_host::interface::*;
use qx86::vm::*;
use neutron_testbench::test_interface::*;

const MAX_GAS:u64 = 10000000;

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = PathBuf::from(&args[1]);
    let file = elf::File::open_path(&path).unwrap();

    let text_scn = file.get_section(".text").unwrap();
    assert!(text_scn.shdr.addr == 0x10000);
    let data_scn = file.get_section(".data").unwrap();
    assert!(data_scn.shdr.addr == 0x80020000);

    let mut api = TestbenchAPI::default();
    setup_api(&mut api, &text_scn.data, &data_scn.data);
    let mut vm:VM = VM::default();
    vm.charger = GasCharger::test_schedule();
    let mut hypervisor = NeutronHypervisor{context: api.get_context(), api: Box::new(api.clone())};
    hypervisor.init_cpu(&mut vm).unwrap();
    hypervisor.create_contract_from_sccs(&mut vm).unwrap();
    let x = vm.execute(&mut hypervisor);
    vm.print_diagnostics();
    println!("Used gas: {}", MAX_GAS - vm.gas_remaining);
    x.unwrap();
}

fn setup_api(api: &mut TestbenchAPI, code: &Vec<u8>, data: &Vec<u8>){
    api.push_sccs(&vec![]).unwrap(); //extra data
    api.push_sccs(data).unwrap();
    api.push_sccs(&vec![1]).unwrap(); //data section count
    api.push_sccs(code).unwrap();
    api.push_sccs(&vec![1]).unwrap(); //code section count
    api.push_sccs(&vec![0, 0, 0, 0]).unwrap(); //vmversion (fill in properly later)
    api.context.exec.gas_limit = MAX_GAS;
}

 