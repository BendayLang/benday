use super::{
	action::{Action, ActionType},
	runner,
};
use crate::exectute::{
	console::Console,
	execute::{execute_node, State},
	load_ast_from, save_ast_to, VariableMap,
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
		let runner_result = execute_node(&ast, &mut variables, id_path, &mut console, &mut actions, &mut states, 0);
		(console, actions)
	} else {
		runner(&ast)
	};

	assert_eq!(expected_stdout.map_or(Vec::new(), |v| v), stdout_result.stdout, "stdout");
	for i in 0..expected_actions.len() {
		assert_eq!(expected_actions[i], actions_result[i], "action {}", i);
	}
	assert_eq!(expected_actions, actions_result, "actions");
}

#[test]
fn should_do_nothing_when_empty_sequence() {
	let ast = Node { id: 0, data: NodeData::Sequence(vec![]) };
}

#[test]
fn should_error_when_root_node_is_not_a_sequence() {
	let ast = Node { id: 0, data: NodeData::RawText("Hello world".to_string()) };
	test(ast, None, None, vec![Action::new(ActionType::Error(models::error::ErrorType::RootIsNotSequence), 0, 0)]);
}

#[test]
fn should_assign_variable_and_print_it() {
	let ast = Node {
		id: 0,
		data: NodeData::Sequence(vec![
			Node {
				id: 1,
				data: NodeData::VariableAssignment(VariableAssignment {
					name_id: 100,
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
			Action::new(ActionType::Entered { from: 0 }, 0, 0),            // -> Sequence
			Action::new(ActionType::Entered { from: 0 }, 0, 1),            // -> VariableAssignment
			Action::new(ActionType::Entered { from: 1 }, 0, 100),          // VariableAssignment::name
			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0, 100), //
			Action::new(ActionType::Entered { from: 100 }, 0, 2),          // VariableAssignment::value
			Action::new(ActionType::EvaluateRawText, 0, 2),                // VariableAssignment::value
			Action::new(ActionType::Return(Ok(Some(ReturnValue::Int(42)))), 0, 2), // VariableAssignment::value -> VariableAssignment
			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) }, 1, 1), // VariableAssignment
			Action::new(ActionType::Return(Ok(None)), 1, 1),               // VariableAssignment -> Sequence
			Action::new(ActionType::Entered { from: 100 }, 1, 3),          // FunctionCall
			Action::new(ActionType::GetArgs, 1, 3),                        // FunctionCall
			Action::new(ActionType::Entered { from: 3 }, 1, 4),            // FunctionCall::argv[0]
			Action::new(ActionType::EvaluateRawText, 1, 4),                // FunctionCall::argv[0]
			Action::new(ActionType::Return(Ok(Some(ReturnValue::Int(42)))), 1, 4), // FunctionCall::argv[0] -> FunctionCall
			Action::new(ActionType::CallBuildInFn("print".to_string()), 1, 3), // FunctionCall
			Action::new(ActionType::PushStdout("42".to_string()), 1, 3),   // FunctionCall
			Action::new(ActionType::Return(Ok(None)), 1, 3),               // FunctionCall -> Sequence
			Action::new(ActionType::Return(Ok(None)), 0, 0),               // Sequence ->
		],
	);
}

// #[test]
// fn should_return_value_when_raw_text() {
// 	let ast = Node { id: 0, data: NodeData::RawText("42".to_string()) };
// 	test(
// 		ast,
// 		Some(HashMap::new()),
// 		None,
// 		vec![
// 			Action::new(ActionType::EvaluateRawText, 0, 0),
// 			Action::new(ActionType::Return(Ok(Some(ReturnValue::Int(42)))), 0, 0),
// 		],
// 	);
// }

// #[test]
// fn should_return_when_raw_text_and_stop_there() {
// 	let ast = Node {
// 		id: 0,
// 		data: NodeData::Sequence(vec![
// 			Node { id: 1, data: NodeData::RawText("42".to_string()) },
// 			Node { id: 2, data: NodeData::RawText("24".to_string()) },
// 			Node {
// 				id: 3,
// 				data: NodeData::FunctionCall(FunctionCall {
// 					name: "print".to_string(),
// 					argv: vec![Node { id: 4, data: NodeData::RawText("42".to_string()) }],
// 				}),
// 			},
// 		]),
// 	};
// 	test(
// 		ast,
// 		None,
// 		None,
// 		vec![
// 			Action::new(ActionType::EvaluateRawText, 0, 1),
// 			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0, 1),
// 			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0, 0),
// 		],
// 	);
// }

// #[test]
// fn should_print_raw_text() {
// 	let ast = Node {
// 		id: 0,
// 		data: NodeData::Sequence(vec![Node {
// 			id: 1,
// 			data: NodeData::FunctionCall(FunctionCall {
// 				name: "print".to_string(),
// 				argv: vec![Node { id: 2, data: NodeData::RawText("42".to_string()) }],
// 			}),
// 		}]),
// 	};
// 	test(
// 		ast,
// 		None,
// 		Some(vec!["42".to_string()]),
// 		vec![
// 			Action::new(ActionType::GetArgs, 0, 1),
// 			Action::new(ActionType::EvaluateRawText, 0, 2),
// 			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0, 2),
// 			Action::new(ActionType::CallBuildInFn("print".to_string()), 0, 1),
// 			Action::new(ActionType::PushStdout("42".to_string()), 0, 1),
// 			Action::new(ActionType::Return(Ok(None)), 0, 1),
// 			Action::new(ActionType::Return(Ok(None)), 0, 0),
// 		],
// 	);
// }

// #[test]
// fn should_print_variable_in_a_while_loop() {
// 	let ast = Node {
// 		id: 0,
// 		data: NodeData::While(While {
// 			is_do: false,
// 			condition: Box::new(Node { id: 1, data: NodeData::RawText("{x} < 2".to_string()) }),
// 			sequence: Box::new(Node {
// 				id: 200,
// 				data: NodeData::Sequence(vec![
// 					Node {
// 						id: 4,
// 						data: NodeData::FunctionCall(FunctionCall {
// 							name: "print".to_string(),
// 							argv: vec![Node { id: 5, data: NodeData::RawText("x={x} !".to_string()) }],
// 						}),
// 					},
// 					Node {
// 						id: 2,
// 						data: NodeData::VariableAssignment(VariableAssignment {
// 							name_id: 201,
// 							name: "x".to_string(),
// 							value: Box::new(Node { id: 3, data: NodeData::RawText("{x} + 1".to_string()) }),
// 						}),
// 					},
// 				]),
// 			}),
// 		}),
// 	};
// 	test(
// 		ast,
// 		Some(HashMap::from([(("x".to_string(), 0), ReturnValue::Int(0))])),
// 		Some(vec!["x=0 !".to_string(), "x=1 !".to_string()]),
// 		vec![
// 			Action::new(ActionType::ControlFlowEvaluateCondition, 0, 0),
// 			Action::new(ActionType::EvaluateRawText, 0, 0),
// 			Action::new(ActionType::return_some(ReturnValue::Int(1)), 0, 0),
// 			Action::new(ActionType::GetArgs, 0, 0),
// 			Action::new(ActionType::EvaluateRawText, 0, 0),
// 			Action::new(ActionType::return_some(ReturnValue::String("x=0 !".to_string())), 0, 0),
// 			Action::new(ActionType::CallBuildInFn("print".to_string()), 0, 0),
// 			Action::new(ActionType::PushStdout("x=0 !".to_string()), 0, 0),
// 			Action::new(ActionType::Return(Ok(None)), 0, 0),
// 			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0, 0),
// 			Action::new(ActionType::EvaluateRawText, 0, 0),
// 			Action::new(ActionType::return_some(ReturnValue::Int(1)), 0, 0),
// 			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(1) }, 1, 0),
// 			Action::new(ActionType::Return(Ok(None)), 1, 0),
// 			Action::new(ActionType::ControlFlowEvaluateCondition, 1, 0),
// 			Action::new(ActionType::EvaluateRawText, 1, 0),
// 			Action::new(ActionType::return_some(ReturnValue::Int(1)), 1, 0),
// 			Action::new(ActionType::GetArgs, 1, 0),
// 			Action::new(ActionType::EvaluateRawText, 1, 0),
// 			Action::new(ActionType::return_some(ReturnValue::String("x=1 !".to_string())), 1, 0),
// 			Action::new(ActionType::CallBuildInFn("print".to_string()), 1, 0),
// 			Action::new(ActionType::PushStdout("x=1 !".to_string()), 1, 0),
// 			Action::new(ActionType::Return(Ok(None)), 1, 0),
// 			Action::new(ActionType::CheckVarNameValidity(Ok(())), 1, 0),
// 			Action::new(ActionType::EvaluateRawText, 1, 0),
// 			Action::new(ActionType::return_some(ReturnValue::Int(2)), 1, 0),
// 			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(2) }, 2, 0),
// 			Action::new(ActionType::Return(Ok(None)), 2, 0),
// 			Action::new(ActionType::ControlFlowEvaluateCondition, 2, 0),
// 			Action::new(ActionType::EvaluateRawText, 2, 0),
// 			Action::new(ActionType::return_some(ReturnValue::Int(0)), 2, 0),
// 			Action::new(ActionType::Return(Ok(None)), 2, 0),
// 		],
// 	);
// }

// #[test]
// fn sould_reassign_variable_if_condition_is_true() {
// 	let ast = Node {
// 		id: 0,
// 		data: NodeData::Sequence(vec![
// 			Node {
// 				id: 1,
// 				data: NodeData::VariableAssignment(VariableAssignment {
// 					name_id: 302,
// 					name: "x".to_string(),
// 					value: Box::new(Node { id: 2, data: NodeData::RawText("10".to_string()) }),
// 				}),
// 			},
// 			Node {
// 				id: 3,
// 				data: NodeData::IfElse(IfElse {
// 					r#if: If {
// 						condition: Box::new(Node { id: 4, data: NodeData::RawText("{x} > 10".to_string()) }),
// 						sequence: Box::new(Node {
// 							id: 200,
// 							data: NodeData::Sequence(vec![Node {
// 								id: 5,
// 								data: NodeData::VariableAssignment(VariableAssignment {
// 									name_id: 303,
// 									name: "x".to_string(),
// 									value: Box::new(Node { id: 6, data: NodeData::RawText("{x} + 1".to_string()) }),
// 								}),
// 							}]),
// 						}),
// 					},
// 					elif: Some(vec![If {
// 						condition: Box::new(Node { id: 7, data: NodeData::RawText("{x} > 20".to_string()) }),
// 						sequence: Box::new(Node {
// 							id: 201,
// 							data: NodeData::Sequence(vec![Node {
// 								id: 8,
// 								data: NodeData::VariableAssignment(VariableAssignment {
// 									name_id: 304,
// 									name: "x".to_string(),
// 									value: Box::new(Node { id: 9, data: NodeData::RawText("{x} + 2".to_string()) }),
// 								}),
// 							}]),
// 						}),
// 					}]),
// 					r#else: Some(Box::new(Node {
// 						id: 202,
// 						data: NodeData::Sequence(vec![Node {
// 							id: 11,
// 							data: NodeData::VariableAssignment(VariableAssignment {
// 								name_id: 305,
// 								name: "x".to_string(),
// 								value: Box::new(Node { id: 12, data: NodeData::RawText("{x} + 3".to_string()) }),
// 							}),
// 						}]),
// 					})),
// 				}),
// 			},
// 		]),
// 	};
// 	test(
// 		ast,
// 		None,
// 		None,
// 		vec![
// 			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0),
// 			Action::new(ActionType::EvaluateRawText, 0),
// 			Action::new(ActionType::return_some(ReturnValue::Int(10)), 0),
// 			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(10) }, 1),
// 			Action::new(ActionType::Return(Ok(None)), 1),
// 			Action::new(ActionType::ControlFlowEvaluateCondition, 1),
// 			Action::new(ActionType::EvaluateRawText, 1),
// 			Action::new(ActionType::return_some(ReturnValue::Int(0)), 1),
// 			Action::new(ActionType::ControlFlowEvaluateCondition, 1),
// 			Action::new(ActionType::EvaluateRawText, 1),
// 			Action::new(ActionType::return_some(ReturnValue::Int(0)), 1),
// 			Action::new(ActionType::CheckVarNameValidity(Ok(())), 1),
// 			Action::new(ActionType::EvaluateRawText, 1),
// 			Action::new(ActionType::Return(Ok(Some(ReturnValue::Int(13)))), 1),
// 			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(13) }, 2),
// 			Action::new(ActionType::Return(Ok(None)), 2),
// 			Action::new(ActionType::Return(Ok(None)), 2),
// 			Action::new(ActionType::Return(Ok(None)), 2),
// 		],
// 	);
// }

// #[test]
// fn should_return_math_expression_result() {
// 	let ast = Node {
// 		id: 0,
// 		data: NodeData::Sequence(vec![
// 			Node {
// 				id: 1,
// 				data: NodeData::VariableAssignment(VariableAssignment {
// 					name: "x".to_string(),
// 					value: Box::new(Node { id: 2, data: NodeData::RawText("42".to_string()) }),
// 				}),
// 			},
// 			Node { id: 3, data: NodeData::RawText("2 + 2 - {x}".to_string()) },
// 		]),
// 	};
// 	test(
// 		ast,
// 		None,
// 		None,
// 		vec![
// 			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0),
// 			Action::new(ActionType::EvaluateRawText, 0),
// 			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0),
// 			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) }, 1),
// 			Action::new(ActionType::Return(Ok(None)), 1),
// 			Action::new(ActionType::EvaluateRawText, 1),
// 			Action::new(ActionType::return_some(ReturnValue::Int(-38)), 1),
// 			Action::new(ActionType::return_some(ReturnValue::Int(-38)), 1),
// 		],
// 	);
// }

// #[test]
// fn should_reassign_variable() {
// 	let ast = Node {
// 		id: 0,
// 		data: NodeData::Sequence(vec![
// 			Node {
// 				id: 1,
// 				data: NodeData::VariableAssignment(VariableAssignment {
// 					name: "x".to_string(),
// 					value: Box::new(Node { id: 2, data: NodeData::RawText("42".to_string()) }),
// 				}),
// 			},
// 			Node {
// 				id: 3,
// 				data: NodeData::VariableAssignment(VariableAssignment {
// 					name: "x".to_string(),
// 					value: Box::new(Node { id: 4, data: NodeData::RawText("24".to_string()) }),
// 				}),
// 			},
// 		]),
// 	};
// 	test(
// 		ast,
// 		None,
// 		None,
// 		vec![
// 			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0),
// 			Action::new(ActionType::EvaluateRawText, 0),
// 			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0),
// 			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) }, 1),
// 			Action::new(ActionType::Return(Ok(None)), 1),
// 			Action::new(ActionType::CheckVarNameValidity(Ok(())), 1),
// 			Action::new(ActionType::EvaluateRawText, 1),
// 			Action::new(ActionType::return_some(ReturnValue::Int(24)), 1),
// 			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(24) }, 2),
// 			Action::new(ActionType::Return(Ok(None)), 2),
// 			Action::new(ActionType::Return(Ok(None)), 2),
// 		],
// 	);
// }

// #[test]
// fn should_reassign_variable_and_keep_original_scope() {
// 	let ast = Node {
// 		id: 0,
// 		data: NodeData::Sequence(vec![
// 			Node {
// 				id: 1,
// 				data: NodeData::VariableAssignment(VariableAssignment {
// 					name: "x".to_string(),
// 					value: Box::new(Node { id: 2, data: NodeData::RawText("42".to_string()) }),
// 				}),
// 			},
// 			Node {
// 				id: 3,
// 				data: NodeData::Sequence(vec![Node {
// 					id: 4,
// 					data: NodeData::VariableAssignment(VariableAssignment {
// 						name: "x".to_string(),
// 						value: Box::new(Node { id: 5, data: NodeData::RawText("24".to_string()) }),
// 					}),
// 				}]),
// 			},
// 		]),
// 	};
// 	test(
// 		ast,
// 		None,
// 		None,
// 		vec![
// 			Action::new(ActionType::CheckVarNameValidity(Ok(())), 0),
// 			Action::new(ActionType::EvaluateRawText, 0),
// 			Action::new(ActionType::return_some(ReturnValue::Int(42)), 0),
// 			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(42) }, 1),
// 			Action::new(ActionType::Return(Ok(None)), 1),
// 			Action::new(ActionType::CheckVarNameValidity(Ok(())), 1),
// 			Action::new(ActionType::EvaluateRawText, 1),
// 			Action::new(ActionType::return_some(ReturnValue::Int(24)), 1),
// 			Action::new(ActionType::AssignVariable { key: ("x".to_string(), 0), value: ReturnValue::Int(24) }, 2),
// 			Action::new(ActionType::Return(Ok(None)), 2),
// 			Action::new(ActionType::Return(Ok(None)), 2),
// 			Action::new(ActionType::Return(Ok(None)), 2),
// 		],
// 	);
// }
