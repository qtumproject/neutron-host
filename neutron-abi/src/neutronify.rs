extern crate proc_macro;

use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn neutronify(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    //println!("attr: \"{}\"", _attr.to_string());
    println!("item: \"{}\"", item.to_string());
    let func = parse_macro_input!(item as ItemFn);
	let input_funcs = func.sig.inputs
			.into_iter()
			.map(|item| println!("{}", item.to_string()))
			.collect()
	
    input_funcs
}