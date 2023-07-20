use models::{
	ast::{Id, Node},
	return_value::ReturnValue,
	runner::{IdPath, VariableMap},
};
use std::collections::HashMap;

pub fn find_variable(expression: &str, variables: &VariableMap, id_path: &IdPath) -> Option<(ReturnValue, Id)> {
	for id in id_path.iter().rev() {
		let variable_key = (expression.to_string(), *id);
		if variables.contains_key(&variable_key) {
			let value = variables.get(&variable_key).unwrap().clone();
			return Some((value, *id));
		}
	}
	None
}
