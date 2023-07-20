use models::runner::{IdPath, VariableMap};
use models::{
	ast::{Id, Node},
	error::VariableExpansionError,
	return_value::ReturnValue,
};
use std::collections::HashMap;

pub fn expand_variables(expression: &str, variables: &VariableMap, id_path: &IdPath) -> Result<String, VariableExpansionError> {
	let start = expression.find('{');
	let end = expression.find('}');
	if start == None {
		if end == None {
			return Ok(expression.to_string());
		}
		return Err(VariableExpansionError::MissingOpeningBracket);
	} else if end == None {
		return Err(VariableExpansionError::MissingClosingBracket);
	}
	let start = start.unwrap();
	let end = end.unwrap();
	if start > end {
		return Err(VariableExpansionError::BracketOrder);
	}
	let variable_name = &expression[start + 1..end];
	let value_option: Option<(ReturnValue, Id)> = crate::find_variable::find_variable(variable_name, variables, id_path);
	match value_option {
		Some((value, _)) => {
			let result = expression[0..start].to_string() + &value.to_string() + &expression[end + 1..];
			return expand_variables(&result, variables, id_path);
		}
		None => {
			return Err(VariableExpansionError::VariableNotFound(variable_name.to_string()));
		}
	}
}

#[cfg(test)]
mod expand_variables_test {
	use super::*;
	#[test]
	fn basic() {
		let var: VariableMap = HashMap::from([(("name".to_string(), 0), ReturnValue::String("michelle".to_string()))]);
		let res = expand_variables("salut {name}, votre nom c'est bien {name} n'est-ce pas ??", &var, &vec![0]);

		assert_eq!(res, Ok("salut michelle, votre nom c'est bien michelle n'est-ce pas ??".to_string()));
	}

	#[test]
	fn age_de_bob() {
		assert_eq!(
			expand_variables(
				"{age de Bob} > 12",
				&HashMap::from([(("age de Bob".to_string(), 0), ReturnValue::Int(7)),]),
				&vec![0],
			),
			Ok("7 > 12".to_string())
		);
	}

	#[test]
	fn fail_unclosed() {
		assert_eq!(
			expand_variables("salut {unclosed   ", &HashMap::new(), &Vec::new()),
			Err(VariableExpansionError::MissingClosingBracket)
		);
		assert_eq!(
			expand_variables("salut }unclosed   ", &HashMap::new(), &Vec::new()),
			Err(VariableExpansionError::MissingOpeningBracket)
		);
		assert_eq!(
			expand_variables("{salut unclosed   ", &HashMap::new(), &Vec::new()),
			Err(VariableExpansionError::MissingClosingBracket)
		);
		assert_eq!(
			expand_variables("}{salut unclosed   ", &HashMap::new(), &Vec::new()),
			Err(VariableExpansionError::BracketOrder)
		);
	}
}
