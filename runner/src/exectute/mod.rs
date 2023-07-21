mod execute;
mod user_prefs {
	pub const MAX_ITERATION: usize = 100;
}
#[cfg(test)]
mod tests;

use self::execute::Action;
use crate::math;
use execute::execute_node;
use models::{
	ast,
	error::ErrorMessage,
	return_value::ReturnValue,
	runner::{AstResult, IdPath, VariableMap},
};
use std::collections::HashMap;

pub fn runner(ast: &ast::Node) -> (AstResult, Vec<String>, VariableMap, Vec<Action>) {
	match &ast.data {
		models::ast::NodeData::Sequence(_) => (),
		_ => {
			return (
				Err(vec![ErrorMessage::new(vec![], models::error::ErrorType::RootIsNotSequence, None)]),
				Vec::new(),
				HashMap::new(),
				Vec::new(),
			)
		}
	}
	let mut variables: VariableMap = HashMap::new();
	let mut stdout = Vec::new();
	let mut id_path: IdPath = Vec::new();
	let mut actions: Vec<Action> = Vec::new();
	let return_value: Result<Option<ReturnValue>, Vec<ErrorMessage>> =
		execute_node(ast, &mut variables, &mut id_path, &mut stdout, &mut actions);

	(return_value, stdout, variables, actions)
}

pub fn linter(_ast: &ast::Node) -> AstResult {
	todo!("Implement linter (after runner, which has the exact same logic)")
}
