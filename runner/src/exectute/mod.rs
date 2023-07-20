mod execute;
#[cfg(test)]
mod tests;

use crate::math;
use execute::execute_node;
use models::{
	ast,
	error::ErrorMessage,
	return_value::ReturnValue,
	runner::{AstResult, IdPath, VariableMap},
};
use std::collections::HashMap;

use self::execute::Action;
mod user_prefs {
	pub const MAX_ITERATION: usize = 100;
}

pub struct Runner {
	ast: ast::Node,
	variables: VariableMap,
	id_path: IdPath,
	stdout: Vec<String>,
}

impl Default for Runner {
	fn default() -> Self {
		Runner {
			ast: ast::Node { id: 0, data: models::ast::NodeData::Sequence(Vec::new()) },
			variables: HashMap::new(),
			id_path: Vec::new(),
			stdout: Vec::new(),
		}
	}
}

impl Runner {
	pub fn new(ast: ast::Node) -> Self {
		Runner { ast, ..Default::default() }
	}
}

// pub enum RunningRythme {
// 	All,
// 	OneByOne,
// }

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
	// let return_value: AstResult = match running_rythme {
	// 	RunningRythme::All => execute_node(ast, &mut variables, &mut Vec::new(), &mut stdout),
	// 	RunningRythme::OneByOne { mut id_path } => {
	// 		execute_node_one_by_one(ast, &mut variables, &mut id_path, &mut stdout, RunningRythme::OneByOne { id_path })
	// 	}
	// };
	let mut id_path: IdPath = Vec::new();
	let mut actions: Vec<Action> = Vec::new();
	let return_value: Result<Option<ReturnValue>, Vec<ErrorMessage>> =
		execute_node(ast, &mut variables, &mut id_path, &mut stdout, &mut actions);

	return (return_value, stdout, variables, actions); // TODO est-ce que je return aussi le id path ?
}

pub fn linter(_ast: &ast::Node) -> AstResult {
	todo!("Implement linter")
}
