use ramhorns::{Template, Content};
use serde_derive::Deserialize;

#[derive(Content, Deserialize)]
pub struct ContractDefinitions {
	pub name: String,
	pub implements: Vec<String>,
	pub function: Vec<Function>,
}

#[derive(Content, Deserialize)]
pub struct Function {
	pub name: String,
	pub inputs: Vec<Ty>, 
	pub outputs: Vec<Ty>,
}

#[derive(Content, Deserialize)]
pub struct Ty {
	pub name: String,
	pub r#type: String,
}

pub fn fill_template(source: &ContractDefinitions) -> String {
	let tmpl_code = "pub trait {{name}} { \n{{#function}} #[neutronify]\n pub fn {{name}}(&self{{#inputs}}, {{name}}: {{r#type}}{{/inputs}}) {{#outputs}} -> {{r#type}}{{/outputs}} {}\n {{/function}} }";
	let tmpl = Template::new(tmpl_code).unwrap();
	return tmpl.render(source);
}

impl ContractDefinitions {
	pub fn process_types(&mut self) {
		for func in &mut self.function {
			for ty in &mut func.inputs {
				ty.r#type = match_types(ty.r#type.clone());
			}
			for ty in &mut func.outputs {
				ty.r#type = match_types(ty.r#type.clone());
			}
		}
	}
}

pub fn match_types(ty: String) -> String {
	let fixed_ty = match ty.as_str() {
		"uint8" | "u8" => "u8".to_string(),
		"uint16" | "u16" => "u16".to_string(),
		"uint32" | "u32" => "u32".to_string(),
		"uint64" | "u64" => "u64".to_string(),
		"int8" | "i8" => "i8".to_string(),
		"int16" | "i16" => "i16".to_string(),
		"int32" | "i32" => "i32".to_string(),
		"int64" | "i64" => "i64".to_string(),
		"string" | "String" => "String".to_string(),
		"address" | "NeutronAddress" => "NeutronAddress".to_string(),
		_ => panic!("invalid type proposed")
	};
	return fixed_ty;
}