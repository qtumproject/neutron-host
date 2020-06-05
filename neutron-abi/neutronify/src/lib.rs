extern crate proc_macro;
use self::proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};
//use neutron_star::syscalls::*;

#[proc_macro_attribute]
pub fn neutronify(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    //println!("attr: \"{}\"", _attr.to_string());
    //let item = proc_macro2::TokenStream::from(input);
    println!("item: \"{}\"", item.to_string());
    let func = parse_macro_input!(item as ItemFn);
	let input_funcs = func.sig.inputs
			.iter()
			.map(|item| println!("{:?}", try_me(item)));
	
    //input_funcs
    attr
}

fn try_me(arg: &syn::FnArg) -> String {
    match arg {
        syn::FnArg::Receiver(_r) => "is receiver".to_string(),
        syn::FnArg::Typed(_p) => "is pattern type".to_string()
    }
}