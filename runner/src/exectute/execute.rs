use super::{math, user_prefs};
use crate::{find_variable::find_variable, variables_expansion::expand_variables};
use models::{
	ast::*,
	error::{ErrorMessage, ErrorType, VariableExpansionError},
	return_value::ReturnValue,
};
use models::{
	error,
	runner::{AstResult, IdPath, VariableMap},
};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, PartialEq)]
pub enum Action {
	Goto(Id),
	Return(AstResult),
	CheckVarNameValidity(bool),
	EvaluateRawText,
	AssigneVariable { key: (String, Id), value: ReturnValue },
	CallBuildInFn(String),
	PushStdout(String),
	GetArgs,
	ControlFlowEvaluateCondition,
}

impl Action {
	pub fn return_some(return_value: ReturnValue) -> Self {
		Self::Return(Ok(Some(return_value)))
	}
}

pub fn execute_node(
	ast: &Node, variables: &mut VariableMap, id_path: &mut IdPath, stdout: &mut Vec<String>, actions: &mut Vec<Action>,
) -> AstResult {
	id_path.push(ast.id);
	actions.push(Action::Goto(ast.id));
	let res: AstResult = match &ast.data {
		NodeData::Sequence(sequence) => handle_sequence(sequence, variables, id_path, stdout, actions),
		NodeData::While(while_node) => handle_while(while_node, variables, id_path, stdout, actions),
		NodeData::IfElse(ifelse) => handle_if_else(ifelse, variables, id_path, stdout, actions),
		NodeData::RawText(value) => handle_raw_text(value, variables, id_path, actions),
		NodeData::VariableAssignment(variable_assignment) => {
			handle_variable_assignment(variable_assignment, variables, id_path, stdout, actions)
		}
		NodeData::FunctionCall(function_call) => handle_function_call(function_call, variables, id_path, stdout, actions),
		NodeData::FunctionDeclaration(fn_declaration) => handle_function_declaration(fn_declaration),
	};
	if id_path.pop() != Some(ast.id) {
		panic!("Id path is not correct");
	}
	actions.push(Action::Return(res.clone())); // TODO ne pas clone le result a chaque fois ?
	res
}

fn handle_function_declaration(_fn_declaration: &FunctionDeclaration) -> AstResult {
	unimplemented!();
}

fn handle_while(
	while_node: &While, variables: &mut VariableMap, id_path: &mut IdPath, stdout: &mut Vec<String>, actions: &mut Vec<Action>,
) -> AstResult {
	if while_node.is_do {
		todo!("Implement at the end of the project");
	}
	let mut iteration = 0;
	while {
		actions.push(Action::ControlFlowEvaluateCondition);
		match get_bool(execute_node(&while_node.condition, variables, id_path, stdout, actions)?) {
			Err(err) => return Err(vec![ErrorMessage::new(id_path.clone(), err, None)]),
			Ok(v) => v,
		}
	} {
		let return_value = execute_node(&while_node.sequence, variables, id_path, stdout, actions)?;
		if return_value.is_some() {
			return Ok(return_value);
		}
		iteration += 1;
		if iteration == user_prefs::MAX_ITERATION {
			todo!("break on max iteration ({})", user_prefs::MAX_ITERATION);
		}
	}
	Ok(None)
}

fn handle_if_else(
	ifelse: &IfElse, variables: &mut VariableMap, id_path: &mut IdPath, stdout: &mut Vec<String>, actions: &mut Vec<Action>,
) -> AstResult {
	let res = {
		actions.push(Action::ControlFlowEvaluateCondition);
		match get_bool(execute_node(&ifelse.r#if.condition, variables, id_path, stdout, actions)?) {
			Err(err) => return Err(vec![ErrorMessage::new(id_path.clone(), err, None)]),
			Ok(v) => v,
		}
	};
	if res {
		return execute_node(&ifelse.r#if.sequence, variables, id_path, stdout, actions);
	}
	if let Some(elifs) = &ifelse.elif {
		for elif in elifs {
			let res = {
				actions.push(Action::ControlFlowEvaluateCondition);
				match get_bool(execute_node(&elif.condition, variables, id_path, stdout, actions)?) {
					Err(err) => return Err(vec![ErrorMessage::new(id_path.clone(), err, None)]),
					Ok(v) => v,
				}
			};
			if res {
				return execute_node(&elif.sequence, variables, id_path, stdout, actions);
			}
		}
	}
	if let Some(else_) = &ifelse.r#else {
		return execute_node(else_, variables, id_path, stdout, actions);
	}
	Ok(None)
}

fn handle_raw_text(text: &str, variables: &mut VariableMap, id_path: &IdPath, actions: &mut Vec<Action>) -> AstResult {
	actions.push(Action::EvaluateRawText);
	match expand_variables(text, variables, id_path) {
		Ok(string) => match math::get_math_parsibility(&string) {
			math::MathParsability::Unparsable => Ok(Some(ReturnValue::String(string))),
			_ => match math::math_expression(&string) {
				Ok(v) => Ok(Some(v)),
				Err(_) => todo!(),
			},
		},
		Err(err) => Err(vec![ErrorMessage::new(id_path.clone(), error::ErrorType::VariableExpansionError(err), None)]),
	}
}

fn is_var_name_valid(name: &str) -> bool {
	// TODOO
	true
}

fn handle_variable_assignment(
	variable_assignment: &VariableAssignment, variables: &mut VariableMap, id_path: &mut IdPath, stdout: &mut Vec<String>,
	actions: &mut Vec<Action>,
) -> AstResult {
	let name_validity = is_var_name_valid(&variable_assignment.name);
	actions.push(Action::CheckVarNameValidity(name_validity));
	if !name_validity {
		todo!("name invalid");
	}

	let value = execute_node(&variable_assignment.value, variables, id_path, stdout, actions)?;
	let id = match find_variable(variable_assignment.name.as_str(), variables, id_path) {
		Some((_, id)) => id,
		None => {
			match id_path.len() {
				0 => panic!("the id path is empty"),
				1 => panic!("there is only one element in the id path: '{}'", id_path[0]),
				_ => {}
			}
			// on est sur une nouvelle variable
			// on prend l'avant dernier id du path, car le dernier est celui de la variable et on veut celui du scope
			*(id_path.get(id_path.len() - 2).unwrap())
		}
	};
	if let Some(value) = value {
		let variable_key: (String, u32) = (variable_assignment.name.to_string(), id);
		actions.push(Action::AssigneVariable { key: variable_key.clone(), value: value.clone() });
		let _ = variables.insert(variable_key, value);
		Ok(None)
	} else {
		todo!("TODO: value returned was None/Void, that cannot be assigned to a var")
	}
}

fn handle_function_call(
	function_call: &FunctionCall, variables: &mut VariableMap, id_path: &mut IdPath, stdout: &mut Vec<String>,
	actions: &mut Vec<Action>,
) -> AstResult {
	actions.push(Action::GetArgs); // TODO est-ce bien nescessaire ??
	let args =
		function_call.argv.iter().map(|arg| execute_node(arg, variables, id_path, stdout, actions)).collect::<Vec<AstResult>>();

	actions.push(Action::CallBuildInFn(function_call.name.clone()));
	match function_call.name.as_str() {
		"print" => {
			for arg in args {
				let to_push = match arg? {
					Some(a) => a.to_string(),
					None => "()".to_string(),
				};
				actions.push(Action::PushStdout(to_push.clone()));
				stdout.push(to_push);
			}
		}
		_ => todo!("FunctionCall {}", function_call.name),
	}

	Ok(None)
}

fn handle_sequence(
	sequence: &[Node], variables: &mut VariableMap, id_path: &mut IdPath, stdout: &mut Vec<String>, actions: &mut Vec<Action>,
) -> AstResult {
	sequence
		.iter()
		.find_map(|node| {
			let return_value = execute_node(node, variables, id_path, stdout, actions);
			if return_value != Ok(None) {
				Some(return_value)
			} else {
				None
			}
		})
		.unwrap_or(Ok(None))
}

fn get_bool(return_value: Option<ReturnValue>) -> Result<bool, ErrorType> {
	let res = if let Some(return_value) = return_value {
		match return_value {
			ReturnValue::Bool(val) => val,
			ReturnValue::String(val) => Err(ErrorType::NEW_TYPE(format!("error should return a bool, not a string ({val})")))?,
			ReturnValue::Int(val) => val != 0,
			ReturnValue::Float(val) => val != 0.0,
		}
	} else {
		return Err(ErrorType::NEW_TYPE("void should not be evaluated".to_string()));
	};
	Ok(res)
}
