use super::{
	action::{Action, ActionType},
	runner,
};
use crate::exectute::{
	console::Console,
	execute::{execute_node, State},
	VariableMap,
};
use models::{
	ast::*,
	error::ErrorMessage,
	return_value::{self, ReturnValue},
	runner::AstResult,
};
use std::{collections::HashMap, env::var};

fn test(ast: Node, basic_variables: Option<VariableMap>, expected_stdout: Option<Vec<String>>, expected_actions: Vec<Action>) {
	let (stdout_result, actions_result) = if let Some(mut variables) = basic_variables {
		let mut console = Console::default();
		let mut actions: Vec<Action> = Vec::new();
		let mut states: Vec<State> = Vec::new();
		let id_path = &mut Vec::new();
		let runner_result = execute_node(&ast, &mut variables, id_path, &mut console, &mut actions, &mut states);
		(console, actions)
	} else {
		runner(&ast)
	};

	assert_eq!(expected_stdout.map_or(Vec::new(), |v| v), stdout_result.stdout, "stdout");
	assert_eq!(expected_actions, actions_result, "actions");
	// assert_eq!(expected_variables.map_or(VariableMap::new(), |v| v), var_result, "variables");
	// assert_eq!(expected_return_value.map_or(Ok(None), |v| v), return_value, "return value");
}

#[test]
fn should_do_nothing_when_empty_sequence() {
	let ast = Node { id: 0, data: NodeData::Sequence(vec![]) };
	test(ast, None, None, vec![Action::new(ActionType::Goto(0), 0), Action::new(ActionType::Return(Ok(None)), 0)]);
}

#[test]
fn should_error_when_root_node_is_not_a_sequence() {
	let ast = Node { id: 0, data: NodeData::RawText("Hello world".to_string()) };
	test(ast, None, None, vec![Action::new(ActionType::Error(models::error::ErrorType::RootIsNotSequence), 0)]);
}

#[test]
fn should_assign_variable_and_print_it() {
	let ast = Node {
		id: 0,
		data: NodeData::Sequence(vec![
			Node {
				id: 1,
				data: NodeData::VariableAssignment(VariableAssignment {
					name: "x".to_string(),
					value: Box::new(Node { id: 2, data: NodeData::RawText("42".to_string()) }),
				}),
			},
			Node {
				id: 3,
				data: NodeData::FunctionCall(FunctionCall {
					name: "print".to_string(),
					argv: vec![Node { id: 4, data: NodeData::RawText("{x}".to_string()) }],
				}),
			},
		]),
	};
	test(
		ast,
		None,
		Some(vec!["42".to_string()]),
		vec![
			Action::new(ActionType::Goto(0), 0),
			Action::new(ActionType::Goto(1), 0),
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0),
			Action::new(ActionType::Goto(2), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::Return(Ok(Some(ReturnValue::Int(42)))), 0),
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) }, 1),
			Action::new(ActionType::Return(Ok(None)), 1),
			Action::new(ActionType::Goto(3), 1),
			Action::new(ActionType::GetArgs, 1),
			Action::new(ActionType::Goto(4), 1),
			Action::new(ActionType::EvaluateRawText, 1),
			Action::new(ActionType::Return(Ok(Some(ReturnValue::Int(42)))), 1),
			Action::new(ActionType::CallBuildInFn("print".to_string()), 1),
			Action::new(ActionType::PushStdout("42".to_string()), 1),
			Action::new(ActionType::Return(Ok(None)), 1),
			Action::new(ActionType::Return(Ok(None)), 1),
		],
	);
}

#[test]
fn should_return_value_when_raw_text() {
	let ast = Node { id: 0, data: NodeData::RawText("42".to_string()) };
	test(
		ast,
		Some(HashMap::new()),
		None,
		vec![
			Action::new(ActionType::Goto(0), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::Return(Ok(Some(ReturnValue::Int(42)))), 0),
		],
	);
}

#[test]
fn should_return_when_raw_text_and_stop_there() {
	let ast = Node {
		id: 0,
		data: NodeData::Sequence(vec![
			Node { id: 1, data: NodeData::RawText("42".to_string()) },
			Node { id: 2, data: NodeData::RawText("24".to_string()) },
			Node {
				id: 3,
				data: NodeData::FunctionCall(FunctionCall {
					name: "print".to_string(),
					argv: vec![Node { id: 4, data: NodeData::RawText("42".to_string()) }],
				}),
			},
		]),
	};
	test(
		ast,
		None,
		None,
		vec![
			Action::new(ActionType::Goto(0), 0),
			Action::new(ActionType::Goto(1), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0),
			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0),
		],
	);
}

#[test]
fn should_print_raw_text() {
	let ast = Node {
		id: 0,
		data: NodeData::Sequence(vec![Node {
			id: 1,
			data: NodeData::FunctionCall(FunctionCall {
				name: "print".to_string(),
				argv: vec![Node { id: 2, data: NodeData::RawText("42".to_string()) }],
			}),
		}]),
	};
	test(
		ast,
		None,
		Some(vec!["42".to_string()]),
		vec![
			Action::new(ActionType::Goto(0), 0),
			Action::new(ActionType::Goto(1), 0),
			Action::new(ActionType::GetArgs, 0),
			Action::new(ActionType::Goto(2), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0),
			Action::new(ActionType::CallBuildInFn("print".to_string()), 0),
			Action::new(ActionType::PushStdout("42".to_string()), 0),
			Action::new(ActionType::Return(Ok(None)), 0),
			Action::new(ActionType::Return(Ok(None)), 0),
		],
	);
}

#[test]
fn should_print_variable_in_a_while_loop() {
	let ast = Node {
		id: 0,
		data: NodeData::While(While {
			is_do: false,
			condition: Box::new(Node { id: 1, data: NodeData::RawText("{x} < 2".to_string()) }),
			sequence: Box::new(Node {
				id: 200,
				data: NodeData::Sequence(vec![
					Node {
						id: 4,
						data: NodeData::FunctionCall(FunctionCall {
							name: "print".to_string(),
							argv: vec![Node { id: 5, data: NodeData::RawText("x={x} !".to_string()) }],
						}),
					},
					Node {
						id: 2,
						data: NodeData::VariableAssignment(VariableAssignment {
							name: "x".to_string(),
							value: Box::new(Node { id: 3, data: NodeData::RawText("{x} + 1".to_string()) }),
						}),
					},
				]),
			}),
		}),
	};
	test(
		ast,
		Some(HashMap::from([(("x".to_string(), 0), ReturnValue::Int(0))])),
		Some(vec!["x=0 !".to_string(), "x=1 !".to_string()]),
		vec![
			Action::new(ActionType::Goto(0), 0),
			Action::new(ActionType::ControlFlowEvaluateCondition, 0),
			Action::new(ActionType::Goto(1), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::return_some(ReturnValue::Int(1)), 0),
			Action::new(ActionType::Goto(200), 0),
			Action::new(ActionType::Goto(4), 0),
			Action::new(ActionType::GetArgs, 0),
			Action::new(ActionType::Goto(5), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::return_some(ReturnValue::String("x=0 !".to_string())), 0),
			Action::new(ActionType::CallBuildInFn("print".to_string()), 0),
			Action::new(ActionType::PushStdout("x=0 !".to_string()), 0),
			Action::new(ActionType::Return(Ok(None)), 0),
			Action::new(ActionType::Goto(2), 0),
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0),
			Action::new(ActionType::Goto(3), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::return_some(ReturnValue::Int(1)), 0),
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(1) }, 1),
			Action::new(ActionType::Return(Ok(None)), 1),
			Action::new(ActionType::Return(Ok(None)), 1),
			Action::new(ActionType::ControlFlowEvaluateCondition, 1),
			Action::new(ActionType::Goto(1), 1),
			Action::new(ActionType::EvaluateRawText, 1),
			Action::new(ActionType::return_some(ReturnValue::Int(1)), 1),
			Action::new(ActionType::Goto(200), 1),
			Action::new(ActionType::Goto(4), 1),
			Action::new(ActionType::GetArgs, 1),
			Action::new(ActionType::Goto(5), 1),
			Action::new(ActionType::EvaluateRawText, 1),
			Action::new(ActionType::return_some(ReturnValue::String("x=1 !".to_string())), 1),
			Action::new(ActionType::CallBuildInFn("print".to_string()), 1),
			Action::new(ActionType::PushStdout("x=1 !".to_string()), 1),
			Action::new(ActionType::Return(Ok(None)), 1),
			Action::new(ActionType::Goto(2), 1),
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 1),
			Action::new(ActionType::Goto(3), 1),
			Action::new(ActionType::EvaluateRawText, 1),
			Action::new(ActionType::return_some(ReturnValue::Int(2)), 1),
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(2) }, 2),
			Action::new(ActionType::Return(Ok(None)), 2),
			Action::new(ActionType::Return(Ok(None)), 2),
			Action::new(ActionType::ControlFlowEvaluateCondition, 2),
			Action::new(ActionType::Goto(1), 2),
			Action::new(ActionType::EvaluateRawText, 2),
			Action::new(ActionType::return_some(ReturnValue::Int(0)), 2),
			Action::new(ActionType::Return(Ok(None)), 2),
		],
	);
}

#[test]
fn sould_reassign_variable_if_condition_is_true() {
	let ast = Node {
		id: 0,
		data: NodeData::Sequence(vec![
			Node {
				id: 1,
				data: NodeData::VariableAssignment(VariableAssignment {
					name: "x".to_string(),
					value: Box::new(Node { id: 2, data: NodeData::RawText("10".to_string()) }),
				}),
			},
			Node {
				id: 3,
				data: NodeData::IfElse(IfElse {
					r#if: If {
						condition: Box::new(Node { id: 4, data: NodeData::RawText("{x} > 10".to_string()) }),
						sequence: Box::new(Node {
							id: 200,
							data: NodeData::Sequence(vec![Node {
								id: 5,
								data: NodeData::VariableAssignment(VariableAssignment {
									name: "x".to_string(),
									value: Box::new(Node { id: 6, data: NodeData::RawText("{x} + 1".to_string()) }),
								}),
							}]),
						}),
					},
					elif: Some(vec![If {
						condition: Box::new(Node { id: 7, data: NodeData::RawText("{x} > 20".to_string()) }),
						sequence: Box::new(Node {
							id: 201,
							data: NodeData::Sequence(vec![Node {
								id: 8,
								data: NodeData::VariableAssignment(VariableAssignment {
									name: "x".to_string(),
									value: Box::new(Node { id: 9, data: NodeData::RawText("{x} + 2".to_string()) }),
								}),
							}]),
						}),
					}]),
					r#else: Some(Box::new(Node {
						id: 202,
						data: NodeData::Sequence(vec![Node {
							id: 11,
							data: NodeData::VariableAssignment(VariableAssignment {
								name: "x".to_string(),
								value: Box::new(Node { id: 12, data: NodeData::RawText("{x} + 3".to_string()) }),
							}),
						}]),
					})),
				}),
			},
		]),
	};
	test(
		ast,
		None,
		None,
		vec![
			Action::new(ActionType::Goto(0), 0),
			Action::new(ActionType::Goto(1), 0),
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0),
			Action::new(ActionType::Goto(2), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::return_some(ReturnValue::Int(10)), 0),
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(10) }, 1),
			Action::new(ActionType::Return(Ok(None)), 1),
			Action::new(ActionType::Goto(3), 1),
			Action::new(ActionType::ControlFlowEvaluateCondition, 1),
			Action::new(ActionType::Goto(4), 1),
			Action::new(ActionType::EvaluateRawText, 1),
			Action::new(ActionType::return_some(ReturnValue::Int(0)), 1),
			Action::new(ActionType::ControlFlowEvaluateCondition, 1),
			Action::new(ActionType::Goto(7), 1),
			Action::new(ActionType::EvaluateRawText, 1),
			Action::new(ActionType::return_some(ReturnValue::Int(0)), 1),
			Action::new(ActionType::Goto(202), 1),
			Action::new(ActionType::Goto(11), 1),
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 1),
			Action::new(ActionType::Goto(12), 1),
			Action::new(ActionType::EvaluateRawText, 1),
			Action::new(ActionType::Return(Ok(Some(ReturnValue::Int(13)))), 1),
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(13) }, 2),
			Action::new(ActionType::Return(Ok(None)), 2),
			Action::new(ActionType::Return(Ok(None)), 2),
			Action::new(ActionType::Return(Ok(None)), 2),
		],
	);
}

#[test]
fn should_return_math_expression_result() {
	let ast = Node {
		id: 0,
		data: NodeData::Sequence(vec![
			Node {
				id: 1,
				data: NodeData::VariableAssignment(VariableAssignment {
					name: "x".to_string(),
					value: Box::new(Node { id: 2, data: NodeData::RawText("42".to_string()) }),
				}),
			},
			Node { id: 3, data: NodeData::RawText("2 + 2 - {x}".to_string()) },
		]),
	};
	test(
		ast,
		None,
		None,
		vec![
			Action::new(ActionType::Goto(0), 0),
			Action::new(ActionType::Goto(1), 0),
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0),
			Action::new(ActionType::Goto(2), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0),
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) }, 1),
			Action::new(ActionType::Return(Ok(None)), 1),
			Action::new(ActionType::Goto(3), 1),
			Action::new(ActionType::EvaluateRawText, 1),
			Action::new(ActionType::return_some(ReturnValue::Int(-38)), 1),
			Action::new(ActionType::return_some(ReturnValue::Int(-38)), 1),
		],
	);
}

#[test]
fn should_reassign_variable() {
	let ast = Node {
		id: 0,
		data: NodeData::Sequence(vec![
			Node {
				id: 1,
				data: NodeData::VariableAssignment(VariableAssignment {
					name: "x".to_string(),
					value: Box::new(Node { id: 2, data: NodeData::RawText("42".to_string()) }),
				}),
			},
			Node {
				id: 3,
				data: NodeData::VariableAssignment(VariableAssignment {
					name: "x".to_string(),
					value: Box::new(Node { id: 4, data: NodeData::RawText("24".to_string()) }),
				}),
			},
		]),
	};
	test(
		ast,
		None,
		None,
		vec![
			Action::new(ActionType::Goto(0), 0),
			Action::new(ActionType::Goto(1), 0),
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0),
			Action::new(ActionType::Goto(2), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0),
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) }, 1),
			Action::new(ActionType::Return(Ok(None)), 1),
			Action::new(ActionType::Goto(3), 1),
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 1),
			Action::new(ActionType::Goto(4), 1),
			Action::new(ActionType::EvaluateRawText, 1),
			Action::new(ActionType::return_some(ReturnValue::Int(24)), 1),
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(24) }, 2),
			Action::new(ActionType::Return(Ok(None)), 2),
			Action::new(ActionType::Return(Ok(None)), 2),
		],
	);
}

#[test]
fn should_reassign_variable_and_keep_original_scope() {
	let ast = Node {
		id: 0,
		data: NodeData::Sequence(vec![
			Node {
				id: 1,
				data: NodeData::VariableAssignment(VariableAssignment {
					name: "x".to_string(),
					value: Box::new(Node { id: 2, data: NodeData::RawText("42".to_string()) }),
				}),
			},
			Node {
				id: 3,
				data: NodeData::Sequence(vec![Node {
					id: 4,
					data: NodeData::VariableAssignment(VariableAssignment {
						name: "x".to_string(),
						value: Box::new(Node { id: 5, data: NodeData::RawText("24".to_string()) }),
					}),
				}]),
			},
		]),
	};
	test(
		ast,
		None,
		None,
		vec![
			Action::new(ActionType::Goto(0), 0),
			Action::new(ActionType::Goto(1), 0),
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0),
			Action::new(ActionType::Goto(2), 0),
			Action::new(ActionType::EvaluateRawText, 0),
			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0),
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) }, 1),
			Action::new(ActionType::Return(Ok(None)), 1),
			Action::new(ActionType::Goto(3), 1),
			Action::new(ActionType::Goto(4), 1),
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 1),
			Action::new(ActionType::Goto(5), 1),
			Action::new(ActionType::EvaluateRawText, 1),
			Action::new(ActionType::return_some(ReturnValue::Int(24)), 1),
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(24) }, 2),
			Action::new(ActionType::Return(Ok(None)), 2),
			Action::new(ActionType::Return(Ok(None)), 2),
			Action::new(ActionType::Return(Ok(None)), 2),
		],
	);
}
