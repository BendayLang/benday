use super::{
	action::{Action, ActionType},
	console::Console,
	math, user_prefs,
};
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
	pub variables: VariableMap,
}

fn push(id: Id, id_path: &mut IdPath, actions: &mut Vec<Action>, states_index: usize, from: Id) {
	actions.push(Action::new(ActionType::Entered { from }, states_index, id));
	id_path.push(id);
}

pub fn execute_node(
	ast: &Node, variables: &mut VariableMap, id_path: &mut IdPath, console: &mut Console, actions: &mut Vec<Action>,
	states: &mut Vec<State>, parent_scope: Id,
) -> AstResult {
	// TODO : separer les erreurs de "Lint" et les erreurs de "Runtime"
	// example : un mauvais nom de variable est une erreur de lint, mais une variable non définie est une erreur de runtime
	// est-ce que les erreurs de lint doivent être traitées ici ?
	// est-ce qu'il faut séparer les erreurs de lint et les erreurs de runtime ?

	if states.is_empty() {
		// at the first iteration, we push the first state (with is probably empty)
		states.push(State { variables: variables.clone() });
	}
	let mut state_index = states.len() - 1;
	let from = match id_path.last() {
		Some(id) => *id,
		None => 0,
	};
	push(ast.id, id_path, actions, state_index, from);
	id_path.push(ast.id);

	let res: AstResult = match &ast.data {
		NodeData::Sequence(sequence) => sequence
			.iter()
			.find_map(|node| {
				let return_value = execute_node(node, variables, id_path, console, actions, states, ast.id);
				if return_value != Ok(None) {
					Some(return_value)
				} else {
					None
				}
			})
			.unwrap_or(Ok(None)),
		NodeData::While(while_node) => {
			let mut res: AstResult = Ok(None);
			if while_node.is_do {
				todo!("Implement at the end of the project");
			}
			let mut iteration = 0;
			while {
				actions.push(Action::new(ActionType::ControlFlowEvaluateCondition, state_index, while_node.sequence.id));
				match get_bool(execute_node(
					&while_node.condition,
					variables,
					id_path,
					console,
					actions,
					states,
					while_node.condition.id,
				)?) {
					Err(err) => return Err(vec![ErrorMessage::new(id_path.clone(), err, None)]),
					Ok(v) => v,
				}
			} {
				let return_value =
					execute_node(&while_node.sequence, variables, id_path, console, actions, states, while_node.sequence.id)?;
				if return_value.is_some() {
					res = Ok(return_value);
					break;
				}
				iteration += 1;
				if iteration == user_prefs::MAX_ITERATION {
					return Err(vec![ErrorMessage::new(
						id_path.clone(),
						error::ErrorType::NewType(format!("Max iteration reached ({})", user_prefs::MAX_ITERATION)),
						None,
					)]);
				}
			}
			res
		}
		NodeData::IfElse(ifelse) => {
			let res = {
				actions.push(Action::new(ActionType::ControlFlowEvaluateCondition, state_index, ifelse.r#if.condition.id));
				match get_bool(execute_node(
					&ifelse.r#if.condition,
					variables,
					id_path,
					console,
					actions,
					states,
					ifelse.r#if.condition.id,
				)?) {
					Err(err) => return Err(vec![ErrorMessage::new(id_path.clone(), err, None)]),
					Ok(v) => v,
				}
			};
			if res {
				execute_node(&ifelse.r#if.sequence, variables, id_path, console, actions, states, ifelse.r#if.sequence.id)
			} else {
				execute_elif(ifelse, variables, id_path, console, actions, states, parent_scope)
			}
		}
		NodeData::RawText(text) => {
			actions.push(Action::new(ActionType::EvaluateRawText, state_index, ast.id));
			match expand_variables(text, variables, id_path) {
				Ok(string) => match math::get_math_parsibility(&string) {
					math::MathParsability::Unparsable => Ok(Some(ReturnValue::String(string))),
					_ => match math::math_expression(&string) {
						Ok(v) => Ok(Some(v)),
						Err(err) => {
							Err(vec![ErrorMessage::new(id_path.clone(), error::ErrorType::MathParsabilityError(err), None)])
						}
					},
				},
				Err(err) => Err(vec![ErrorMessage::new(id_path.clone(), error::ErrorType::VariableExpansionError(err), None)]),
			}
		}
		NodeData::VariableAssignment(variable_assignment) => {
			let name_validity = is_var_name_valid(&variable_assignment.name);
			push(variable_assignment.name_id, id_path, actions, state_index, *id_path.last().unwrap());
			actions.push(Action::new(
				ActionType::CheckVarNameValidity(name_validity.clone()),
				state_index,
				variable_assignment.name_id,
			));
			if name_validity.is_err() {
				Err(vec![ErrorMessage::new(id_path.clone(), name_validity.unwrap_err(), None)])?
			}

			let value = execute_node(
				&variable_assignment.value,
				variables,
				id_path,
				console,
				actions,
				states,
				variable_assignment.value.id,
			)?;
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
			let id = parent_scope;
			if let Some(value) = value {
				let variable_key: (String, u32) = (variable_assignment.name.to_string(), id);
				let _ = variables.insert(variable_key.clone(), value.clone());
				states.push(State { variables: variables.clone() });
				state_index += 1;
				actions.push(Action::new(
					ActionType::AssignVariable { key: variable_key.clone(), value: value.clone() },
					state_index,
					ast.id,
				));
				Ok(None)
			} else {
				Err(vec![ErrorMessage::new(
					id_path.clone(),
					error::ErrorType::NewType("Cannot assign void value to a variable".to_string()),
					None,
				)])
			}
		}
		NodeData::FunctionCall(function_call) => {
			actions.push(Action::new(ActionType::GetArgs, state_index, ast.id));
			let args = function_call
				.argv
				.iter()
				.map(|arg| execute_node(arg, variables, id_path, console, actions, states, arg.id))
				.collect::<Vec<AstResult>>();

			actions.push(Action::new(ActionType::CallBuildInFn(function_call.name.clone()), state_index, ast.id));
			match function_call.name.as_str() {
				"print" => {
					for arg in args {
						let to_push = match arg? {
							Some(a) => a.to_string(),
							None => "()".to_string(),
						};
						actions.push(Action::new(ActionType::PushStdout(to_push.clone()), state_index, ast.id));
						console.stdout.push(to_push);
					}
				}
				_ => Err(vec![ErrorMessage::new(
					id_path.clone(),
					error::ErrorType::NewType(format!("Unknown function '{}'", function_call.name)),
					None,
				)])?,
			}

			Ok(None)
		}
		NodeData::FunctionDeclaration(fn_declaration) => unimplemented!(),
	};

	let poped = id_path.pop();
	if poped != Some(ast.id) {
		// TODO : en gros quand on entre dans une assignation de variable, je ne push pas qu'on entre dedans, du coup, je pop le nom de la variable et pas le bloc entier !
		println!("Id path is not correct. poped: {:?}, ast.id: {}", poped, ast.id);
	}

	actions.push(Action::new(ActionType::Return(res.clone()), state_index, ast.id));
	res
}

fn execute_elif(
	ifelse: &IfElse, variables: &mut VariableMap, id_path: &mut IdPath, console: &mut Console, actions: &mut Vec<Action>,
	states: &mut Vec<State>, parent_scope: Id,
) -> AstResult {
	let state_index = states.len() - 1;
	if let Some(elifs) = &ifelse.elif {
		for elif in elifs {
			let res = {
				actions.push(Action::new(ActionType::ControlFlowEvaluateCondition, state_index, elif.condition.id));
				match get_bool(execute_node(&elif.condition, variables, id_path, console, actions, states, elif.condition.id)?) {
					Err(err) => return Err(vec![ErrorMessage::new(id_path.clone(), err, None)]),
					Ok(v) => v,
				}
			};
			if res {
				return execute_node(&elif.sequence, variables, id_path, console, actions, states, elif.sequence.id);
			}
		}
	}
	if let Some(else_) = &ifelse.r#else {
		return execute_node(else_, variables, id_path, console, actions, states, else_.id);
	}
	Ok(None)
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

	if !name.chars().next().unwrap().is_alphabetic() && !name.starts_with('_') {
		return Err(ErrorType::VariableNameError(error::VariableNameError::InvalidFirstChar));
	}

	Ok(())
}

fn get_bool(return_value: Option<ReturnValue>) -> Result<bool, ErrorType> {
	if let Some(return_value) = return_value {
		return_value.to_bool()
	} else {
		Err(ErrorType::NewType("void should not be evaluated".to_string()))
	}
}
