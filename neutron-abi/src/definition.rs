use ramhorns::{Template, Content};
use serde_derive::Deserialize;

#[derive(Content, Deserialize)]
pub struct ContractDefinitions {
	pub name: String,
	pub implements: Vec<String>,
	pub functions: Vec<Function>,
}

#[derive(Content, Deserialize)]
pub struct Function {
	pub name: String,
	pub inputs: Vec<Ty>, 
	pub outputs: Vec<Ty>,
	#[serde(skip)]
	pub pop_funcs_string: String,
	#[serde(skip)]
	pub push_func_string: String,
	#[serde(skip)]
	pub args_string: String,
}

#[derive(Content, Deserialize)]
pub struct Ty {
	pub name: String,
	pub r#type: String,
	#[serde(skip)]
	pub last: bool,
}

const TEMPLATE_CODE: &str = r#"
	pub trait {{name}} {
		pub fn on_creation(&mut self) -> Result<u32, NeutronError>;
		{{#functions}} pub fn {{name}}(&self{{#inputs}}, {{name}}: {{r#type}}{{/inputs}}) {{#outputs}} -> {{r#type}}{{/outputs}};{{/functions}} 
	}
	impl NeutronContract for {{name}} {
		fn on_creation(&mut self) -> Result<u32, NeutronError> {
        	(&mut self as &mut dyn {{name}}).on_creation()
		}
		fn on_call(&mut self) -> Result<u32, NeutronError> {
        	let __id = pop_sccs_u32()?;
        	match __id {
			{{#functions}}
				functionid!("{{name}}({{#inputs}}{{r#type}}{{^last}},{{/last}}{{/inputs}}){{#outputs}}->{{r#type}}{{/outputs}}") => { 
					{{pop_funcs_string}}
					let result = self.{{name}}({{args_string}})?;
					{{push_func_string}}
				},{{/functions}}
				_ => {
                	revert_execution(ABI_ERROR_BAD_FUNCTION);
            	}
			}
		}
	}

"#;

pub fn fill_template(source: &ContractDefinitions) -> String {
	let tmpl = Template::new(TEMPLATE_CODE).unwrap();
	return tmpl.render(source);
}

impl ContractDefinitions {
	pub fn process_types(&mut self) {
		for func in &mut self.functions {
			let mut pop_funcs: Vec<String> = Vec::new();
			let mut str_args: Vec<String> = Vec::new();
			for (i, ty) in &mut func.inputs.iter_mut().enumerate() {
				ty.r#type = match_types(ty.r#type.clone());
				pop_funcs.push(format!("let __{} = {}?;", i, match_pop_funcs(ty.r#type.clone())).clone());
				str_args.push(format!("__{}", i).clone());
			}

			if let Some(last) = func.inputs.last_mut() {
    			last.last = true;
			}

			func.args_string = str_args.join(",");
			func.pop_funcs_string = pop_funcs.join("\n");
			let mut push_funcs: Vec<String> = Vec::new();
			for ty in &mut func.outputs {
				ty.r#type = match_types(ty.r#type.clone());
				push_funcs.push(format!("{}?;", match_push_funcs(ty.r#type.clone())));
			}
			func.push_func_string = push_funcs.join("\n");
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

pub fn match_pop_funcs(ty: String) -> String {
	let pop_func_str = match ty.as_str() {
		"u8" => "pop_sccs_u8()".to_string(),
		"i8" => "pop_sccs_i8()".to_string(),
		"u16" => "pop_sccs_u16()".to_string(),
		"i16" => "pop_sccs_i16()".to_string(),
		"u32" => "pop_sccs_u32()".to_string(),
		"i32" => "pop_sccs_i32()".to_string(),
		"u64" => "pop_sccs_u64()".to_string(),
		"i64" => "pop_sccs_i64()".to_string(),
		"String" => "pop_sccs_string()".to_string(),
		"NeutronAddress" => "pop_sccs_address()".to_string(),
		_ => panic!("invalid input type proposed for pop funcs")
	};
	return pop_func_str;
}

pub fn match_push_funcs(ty: String) -> String {
	let push_func_str = match ty.as_str() {
		"u8" => "push_sccs_u8(result)".to_string(),
		"i8" => "push_sccs_i8(result)".to_string(),
		"u16" => "push_sccs_u16(result)".to_string(),
		"i16" => "push_sccs_i16(result)".to_string(),
		"u32" => "push_sccs_u32(result)".to_string(),
		"i32" => "push_sccs_i32(result)".to_string(),
		"u64" => "push_sccs_u64(result)".to_string(),
		"i64" => "push_sccs_i64(result)".to_string(),
		"String" => "push_sccs_string(result)".to_string(),
		"NeutronAddress" => "push_sccs_address(result)".to_string(),
		_ => panic!("invalid input type proposed for push funcs")
	};
	return push_func_str;
}