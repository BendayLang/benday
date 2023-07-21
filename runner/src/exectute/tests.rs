use super::runner;
use crate::exectute::{
	execute::{execute_node, Action},
	VariableMap,
};
use models::{
	ast::*,
	error::ErrorMessage,
	return_value::{self, ReturnValue},
	runner::AstResult,
};
use std::{collections::HashMap, env::var};

fn test(
	ast: Node, basic_variables: Option<VariableMap>, expected_stdout: Option<Vec<String>>,
	expected_variables: Option<VariableMap>, expected_return_value: Option<AstResult>, expected_actions: Vec<Action>,
) {
	let (return_value, stdout_result, var_result, actions_result) = if let Some(mut variables) = basic_variables {
		let mut stdout: Vec<String> = Vec::<String>::new();
		let mut actions: Vec<Action> = Vec::new();
		let id_path = &mut Vec::new();
		let runner_result = execute_node(&ast, &mut variables, id_path, &mut stdout, &mut actions);
		(runner_result, stdout, variables, actions)
	} else {
		runner(&ast)
	};

	assert_eq!(expected_stdout.map_or(Vec::new(), |v| v), stdout_result, "stdout");
	assert_eq!(expected_actions, actions_result, "actions");
	assert_eq!(expected_variables.map_or(VariableMap::new(), |v| v), var_result, "variables");
	assert_eq!(expected_return_value.map_or(Ok(None), |v| v), return_value, "return value");
}

#[test]
fn should_do_nothing_when_empty_sequence() {
	let ast = Node { id: 0, data: NodeData::Sequence(vec![]) };
	test(ast, None, None, None, None, vec![Action::Goto(0), Action::Return(Ok(None))]);
}

#[test]
fn should_error_when_root_node_is_not_a_sequence() {
	let ast = Node { id: 0, data: NodeData::RawText("Hello world".to_string()) };
	let error_message = ErrorMessage::new(vec![], models::error::ErrorType::RootIsNotSequence, None);
	test(ast, None, None, None, Some(Err(vec![error_message])), vec![]);
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
		Some(HashMap::from([(("x".to_string(), 0), ReturnValue::Int(42))])),
		Some(Ok(None)),
		vec![
			Action::Goto(0),
			Action::Goto(1),
			Action::CheckVarNameValidity(true),
			Action::Goto(2),
			Action::EvaluateRawText,
			Action::Return(Ok(Some(ReturnValue::Int(42)))),
			Action::AssigneVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) },
			Action::Return(Ok(None)),
			Action::Goto(3),
			Action::GetArgs,
			Action::Goto(4),
			Action::EvaluateRawText,
			Action::Return(Ok(Some(ReturnValue::Int(42)))),
			Action::CallBuildInFn("print".to_string()),
			Action::PushStdout("42".to_string()),
			Action::Return(Ok(None)),
			Action::Return(Ok(None)),
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
		None,
		Some(Ok(Some(ReturnValue::Int(42)))),
		vec![Action::Goto(0), Action::EvaluateRawText, Action::Return(Ok(Some(ReturnValue::Int(42))))],
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
		None,
		Some(Ok(Some(ReturnValue::Int(42)))),
		vec![
			Action::Goto(0),
			Action::Goto(1),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(42)),
			Action::return_some(ReturnValue::Int(42)),
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
		None,
		Some(Ok(None)),
		vec![
			Action::Goto(0),
			Action::Goto(1),
			Action::GetArgs,
			Action::Goto(2),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(42)),
			Action::CallBuildInFn("print".to_string()),
			Action::PushStdout("42".to_string()),
			Action::Return(Ok(None)),
			Action::Return(Ok(None)),
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
		Some(HashMap::from([(("x".to_string(), 0), ReturnValue::Int(2))])),
		Some(Ok(None)),
		vec![
			Action::Goto(0),
			Action::ControlFlowEvaluateCondition,
			Action::Goto(1),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(1)),
			Action::Goto(200),
			Action::Goto(4),
			Action::GetArgs,
			Action::Goto(5),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::String("x=0 !".to_string())),
			Action::CallBuildInFn("print".to_string()),
			Action::PushStdout("x=0 !".to_string()),
			Action::Return(Ok(None)),
			Action::Goto(2),
			Action::CheckVarNameValidity(true),
			Action::Goto(3),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(1)),
			Action::AssigneVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(1) },
			Action::Return(Ok(None)),
			Action::Return(Ok(None)),
			Action::ControlFlowEvaluateCondition,
			Action::Goto(1),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(1)),
			Action::Goto(200),
			Action::Goto(4),
			Action::GetArgs,
			Action::Goto(5),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::String("x=1 !".to_string())),
			Action::CallBuildInFn("print".to_string()),
			Action::PushStdout("x=1 !".to_string()),
			Action::Return(Ok(None)),
			Action::Goto(2),
			Action::CheckVarNameValidity(true),
			Action::Goto(3),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(2)),
			Action::AssigneVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(2) },
			Action::Return(Ok(None)),
			Action::Return(Ok(None)),
			Action::ControlFlowEvaluateCondition,
			Action::Goto(1),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(0)),
			Action::Return(Ok(None)),
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
					if_: If {
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
					else_: Some(Box::new(Node {
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
		Some(HashMap::from([(("x".to_string(), 0), ReturnValue::Int(13))])),
		Some(Ok(None)),
		vec![
			Action::Goto(0),
			Action::Goto(1),
			Action::CheckVarNameValidity(true),
			Action::Goto(2),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(10)),
			Action::AssigneVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(10) },
			Action::Return(Ok(None)),
			Action::Goto(3),
			Action::ControlFlowEvaluateCondition,
			Action::Goto(4),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(0)),
			Action::ControlFlowEvaluateCondition,
			Action::Goto(7),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(0)),
			Action::Goto(202),
			Action::Goto(11),
			Action::CheckVarNameValidity(true),
			Action::Goto(12),
			Action::EvaluateRawText,
			Action::Return(Ok(Some(ReturnValue::Int(13)))),
			Action::AssigneVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(13) },
			Action::Return(Ok(None)),
			Action::Return(Ok(None)),
			Action::Return(Ok(None)),
			Action::Return(Ok(None)),
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
		Some(HashMap::from([(("x".to_string(), 0), ReturnValue::Int(42))])),
		Some(Ok(Some(ReturnValue::Int(-38)))),
		vec![
			Action::Goto(0),
			Action::Goto(1),
			Action::CheckVarNameValidity(true),
			Action::Goto(2),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(42)),
			Action::AssigneVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) },
			Action::Return(Ok(None)),
			Action::Goto(3),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(-38)),
			Action::return_some(ReturnValue::Int(-38)),
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
		Some(HashMap::from([(("x".to_string(), 0), ReturnValue::Int(24))])),
		Some(Ok(None)),
		vec![
			Action::Goto(0),
			Action::Goto(1),
			Action::CheckVarNameValidity(true),
			Action::Goto(2),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(42)),
			Action::AssigneVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) },
			Action::Return(Ok(None)),
			Action::Goto(3),
			Action::CheckVarNameValidity(true),
			Action::Goto(4),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(24)),
			Action::AssigneVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(24) },
			Action::Return(Ok(None)),
			Action::Return(Ok(None)),
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
		Some(HashMap::from([(("x".to_string(), 0), ReturnValue::Int(24))])),
		None,
		vec![
			Action::Goto(0),
			Action::Goto(1),
			Action::CheckVarNameValidity(true),
			Action::Goto(2),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(42)),
			Action::AssigneVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) },
			Action::Return(Ok(None)),
			Action::Goto(3),
			Action::Goto(4),
			Action::CheckVarNameValidity(true),
			Action::Goto(5),
			Action::EvaluateRawText,
			Action::return_some(ReturnValue::Int(24)),
			Action::AssigneVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(24) },
			Action::Return(Ok(None)),
			Action::Return(Ok(None)),
			Action::Return(Ok(None)),
		],
	);
}

// // fn function_declaration() { // TODO
