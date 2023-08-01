pub mod console;
mod execute;
mod user_prefs {
	pub const MAX_ITERATION: usize = 100;
}
#[cfg(test)]
mod tests;

use self::console::Console;
pub use self::execute::ActionType;
use crate::math;
use execute::execute_node;
use models::{
	ast,
	error::ErrorMessage,
	return_value::ReturnValue,
	runner::{AstResult, IdPath, VariableMap},
};
use std::collections::HashMap;

pub fn runner(ast: &ast::Node) -> (Console, Vec<ActionType>) {
	match &ast.data {
		models::ast::NodeData::Sequence(_) => (),
		_ => return (Console::default(), vec![ActionType::Error(models::error::ErrorType::RootIsNotSequence)]),
	}
	let mut variables: VariableMap = HashMap::new();
	let mut console = Console::default();
	let mut id_path: IdPath = Vec::new();
	let mut actions: Vec<ActionType> = Vec::new();
	let return_value: Result<Option<ReturnValue>, Vec<ErrorMessage>> =
		execute_node(ast, &mut variables, &mut id_path, &mut console, &mut actions);

	(console, actions)
}

pub fn linter(_ast: &ast::Node) -> AstResult {
	todo!("Implement linter (after runner, which has the exact same logic)")
}
