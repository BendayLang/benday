use super::{console::Console, math, user_prefs};
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
pub struct State {
	varibles: VariableMap,
}

#[derive(Debug, PartialEq)]
pub struct Action {
	r#type: ActionType,
	new_state: Option<State>,
}

#[derive(Debug, PartialEq)]
pub enum ActionType {
	Goto(Id),
	Return(AstResult),
	CheckVarNameValidity(Result<(), ErrorType>),
	EvaluateRawText,
	AssigneVariable { key: (String, Id), value: ReturnValue },
	CallBuildInFn(String),
	PushStdout(String),
	GetArgs,
	ControlFlowEvaluateCondition,
	Error(ErrorType),
}

impl ActionType {
	pub fn return_some(return_value: ReturnValue) -> Self {
		Self::Return(Ok(Some(return_value)))
	}
}

pub fn execute_node(
	ast: &Node, variables: &mut VariableMap, id_path: &mut IdPath, console: &mut Console, actions: &mut Vec<ActionType>,
) -> AstResult {
	// TODO : separer les erreurs de "Lint" et les erreurs de "Runtime"
	// example : un mauvais nom de variable est une erreur de lint, mais une variable non définie est une erreur de runtime
	// est-ce que les erreurs de lint doivent être traitées ici ?
	// est-ce qu'il faut séparer les erreurs de lint et les erreurs de runtime ?
	id_path.push(ast.id);
	actions.push(ActionType::Goto(ast.id));
	let res: AstResult = match &ast.data {
		NodeData::Sequence(sequence) => handle_sequence(sequence, variables, id_path, console, actions),
		NodeData::While(while_node) => handle_while(while_node, variables, id_path, console, actions),
		NodeData::IfElse(ifelse) => handle_if_else(ifelse, variables, id_path, console, actions),
		NodeData::RawText(value) => handle_raw_text(value, variables, id_path, actions),
		NodeData::VariableAssignment(variable_assignment) => {
			handle_variable_assignment(variable_assignment, variables, id_path, console, actions)
		}
		NodeData::FunctionCall(function_call) => handle_function_call(function_call, variables, id_path, console, actions),
		NodeData::FunctionDeclaration(fn_declaration) => handle_function_declaration(fn_declaration),
	};
	if id_path.pop() != Some(ast.id) {
		panic!("Id path is not correct");
	}
	actions.push(ActionType::Return(res.clone())); // TODO ne pas clone le result a chaque fois ?
	res
}

fn handle_function_declaration(_fn_declaration: &FunctionDeclaration) -> AstResult {
	unimplemented!();
}

fn handle_while(
	while_node: &While, variables: &mut VariableMap, id_path: &mut IdPath, console: &mut Console, actions: &mut Vec<ActionType>,
) -> AstResult {
	if while_node.is_do {
		todo!("Implement at the end of the project");
	}
	let mut iteration = 0;
	while {
		actions.push(ActionType::ControlFlowEvaluateCondition);
		match get_bool(execute_node(&while_node.condition, variables, id_path, console, actions)?) {
			Err(err) => return Err(vec![ErrorMessage::new(id_path.clone(), err, None)]),
			Ok(v) => v,
		}
	} {
		let return_value = execute_node(&while_node.sequence, variables, id_path, console, actions)?;
		if return_value.is_some() {
			return Ok(return_value);
		}
		iteration += 1;
		if iteration == user_prefs::MAX_ITERATION {
			return Err(vec![ErrorMessage::new(
				id_path.clone(),
				error::ErrorType::NEW_TYPE(format!("Max iteration reached ({})", user_prefs::MAX_ITERATION)),
				None,
			)]);
		}
	}
	Ok(None)
}

fn handle_if_else(
	ifelse: &IfElse, variables: &mut VariableMap, id_path: &mut IdPath, console: &mut Console, actions: &mut Vec<ActionType>,
) -> AstResult {
	let res = {
		actions.push(ActionType::ControlFlowEvaluateCondition);
		match get_bool(execute_node(&ifelse.r#if.condition, variables, id_path, console, actions)?) {
			Err(err) => return Err(vec![ErrorMessage::new(id_path.clone(), err, None)]),
			Ok(v) => v,
		}
	};
	if res {
		return execute_node(&ifelse.r#if.sequence, variables, id_path, console, actions);
	}
	if let Some(elifs) = &ifelse.elif {
		for elif in elifs {
			let res = {
				actions.push(ActionType::ControlFlowEvaluateCondition);
				match get_bool(execute_node(&elif.condition, variables, id_path, console, actions)?) {
					Err(err) => return Err(vec![ErrorMessage::new(id_path.clone(), err, None)]),
					Ok(v) => v,
				}
			};
			if res {
				return execute_node(&elif.sequence, variables, id_path, console, actions);
			}
		}
	}
	if let Some(else_) = &ifelse.r#else {
		return execute_node(else_, variables, id_path, console, actions);
	}
	Ok(None)
}

fn handle_raw_text(text: &str, variables: &mut VariableMap, id_path: &IdPath, actions: &mut Vec<ActionType>) -> AstResult {
	actions.push(ActionType::EvaluateRawText);
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

fn is_var_name_valid(name: &str) -> Result<(), ErrorType> {
	// Rules : must be a valid identifier, must not be a keyword
	// - should begin with a letter or an underscore
	// - can contain letters, numbers and underscores
	// - can contain spaces
	// - can't be a keyword

	if name.is_empty() {
		return Err(ErrorType::VariableNameError(error::VariableNameError::Empty));
	}

	if !name.chars().next().unwrap().is_alphabetic() && name.chars().next().unwrap() != '_' {
		return Err(ErrorType::VariableNameError(error::VariableNameError::InvalidFirstChar));
	}

	Ok(())
}

fn handle_variable_assignment(
	variable_assignment: &VariableAssignment, variables: &mut VariableMap, id_path: &mut IdPath, console: &mut Console,
	actions: &mut Vec<ActionType>,
) -> AstResult {
	let name_validity = is_var_name_valid(&variable_assignment.name);
	actions.push(ActionType::CheckVarNameValidity(name_validity.clone()));
	if name_validity.is_err() {
		return Err(vec![ErrorMessage::new(id_path.clone(), name_validity.unwrap_err(), None)]);
	}

	let value = execute_node(&variable_assignment.value, variables, id_path, console, actions)?;
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
		actions.push(ActionType::AssigneVariable { key: variable_key.clone(), value: value.clone() });
		let _ = variables.insert(variable_key, value);
		Ok(None)
	} else {
		todo!("TODO: value returned was None/Void, that cannot be assigned to a var")
	}
}

fn handle_function_call(
	function_call: &FunctionCall, variables: &mut VariableMap, id_path: &mut IdPath, console: &mut Console,
	actions: &mut Vec<ActionType>,
) -> AstResult {
	actions.push(ActionType::GetArgs); // TODO est-ce bien nescessaire ??
	let args =
		function_call.argv.iter().map(|arg| execute_node(arg, variables, id_path, console, actions)).collect::<Vec<AstResult>>();

	actions.push(ActionType::CallBuildInFn(function_call.name.clone()));
	match function_call.name.as_str() {
		"print" => {
			for arg in args {
				let to_push = match arg? {
					Some(a) => a.to_string(),
					None => "()".to_string(),
				};
				actions.push(ActionType::PushStdout(to_push.clone()));
				console.stdout.push(to_push);
			}
		}
		_ => todo!("FunctionCall {}", function_call.name),
	}

	Ok(None)
}

fn handle_sequence(
	sequence: &[Node], variables: &mut VariableMap, id_path: &mut IdPath, console: &mut Console, actions: &mut Vec<ActionType>,
) -> AstResult {
	sequence
		.iter()
		.find_map(|node| {
			let return_value = execute_node(node, variables, id_path, console, actions);
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
