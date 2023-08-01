pub mod action;
pub mod console;
use serde_json;
use std::io::Write;
mod execute;
mod user_prefs {
	pub const MAX_ITERATION: usize = 100;
}
#[cfg(test)]
mod tests;

use self::{
	action::{Action, ActionType},
	console::Console,
	execute::State,
};
use crate::math;
use execute::execute_node;
use models::{
	ast,
	error::ErrorMessage,
	return_value::ReturnValue,
	runner::{AstResult, IdPath, VariableMap},
};
use std::{collections::HashMap, path::Path};

pub fn runner(ast: &ast::Node) -> (Console, Vec<Action>) {
	match &ast.data {
		models::ast::NodeData::Sequence(_) => (),
		_ => {
			return (Console::default(), vec![Action::new(ActionType::Error(models::error::ErrorType::RootIsNotSequence), 0, 0)])
		}
	}

	let mut variables: VariableMap = HashMap::new();
	let mut console = Console::default();
	let mut id_path: IdPath = Vec::new();
	let mut actions: Vec<Action> = Vec::new();
	let mut states: Vec<State> = Vec::new();
	let return_value: Result<Option<ReturnValue>, Vec<ErrorMessage>> =
		execute_node(ast, &mut variables, &mut id_path, &mut console, &mut actions, &mut states, 0);

	(console, actions)
}

pub fn save_ast_to(ast: &ast::Node, path: &Path) -> Result<(), std::io::Error> {
	let mut file = std::fs::File::create(path)?;
	let ast_string = serde_json::to_string_pretty(ast)?;
	file.write_all(ast_string.as_bytes())?;
	Ok(())
}

pub fn load_ast_from(path: &Path) -> Result<ast::Node, std::io::Error> {
	let file = std::fs::File::open(path)?;
	let ast: ast::Node = serde_json::from_reader(file)?;
	Ok(ast)
}
