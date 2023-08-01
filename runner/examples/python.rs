use models::ast;
use runner::transpile::python::ast_to_python;

fn a() -> ast::Node {
	ast::Node {
		id: 0,
		data: ast::NodeData::FunctionCall(ast::FunctionCall {
			name: "print".to_string(),
			argv: vec![
				ast::Node { id: 0, data: ast::NodeData::RawText("Hy".to_string()) },
				ast::Node { id: 0, data: ast::NodeData::RawText("Mom".to_string()) },
			],
		}),
	}
}

fn b() -> ast::Node {
	ast::Node {
		id: 0,
		data: ast::NodeData::Sequence(vec![
			ast::Node {
				id: 0,
				data: ast::NodeData::VariableAssignment(ast::VariableAssignment {
					name_id: 100,
					name: "var".to_string(),
					value: Box::new(ast::Node {
						id: 0,
						data: ast::NodeData::Sequence(vec![
							ast::Node { id: 0, data: ast::NodeData::RawText("1er".to_string()) },
							ast::Node { id: 0, data: ast::NodeData::RawText("2Hy".to_string()) },
						]),
					}),
				}),
			},
			ast::Node {
				id: 0,
				data: ast::NodeData::FunctionCall(ast::FunctionCall {
					name: "print".to_string(),
					argv: vec![
						ast::Node { id: 0, data: ast::NodeData::RawText("1Hy".to_string()) },
						ast::Node {
							id: 0,
							data: ast::NodeData::FunctionCall(ast::FunctionCall {
								name: "print".to_string(),
								argv: vec![
									ast::Node { id: 0, data: ast::NodeData::RawText("2Hy".to_string()) },
									ast::Node { id: 0, data: ast::NodeData::RawText("3Mom".to_string()) },
									ast::Node { id: 0, data: ast::NodeData::RawText("SONTEU".to_string()) },
									ast::Node { id: 0, data: ast::NodeData::RawText("FLEXItarien".to_string()) },
								],
							}),
						},
					],
				}),
			},
			ast::Node {
				id: 0,
				data: ast::NodeData::FunctionCall(ast::FunctionCall {
					name: "print".to_string(),
					argv: vec![
						ast::Node { id: 0, data: ast::NodeData::RawText("4Hy".to_string()) },
						ast::Node { id: 0, data: ast::NodeData::RawText("5Mom".to_string()) },
					],
				}),
			},
			ast::Node {
				id: 0,
				data: ast::NodeData::FunctionCall(ast::FunctionCall {
					name: "print".to_string(),
					argv: vec![
						ast::Node { id: 0, data: ast::NodeData::RawText("6Hy".to_string()) },
						ast::Node { id: 0, data: ast::NodeData::RawText("7Mom".to_string()) },
					],
				}),
			},
			ast::Node {
				id: 0,
				data: ast::NodeData::FunctionCall(ast::FunctionCall {
					name: "print".to_string(),
					argv: vec![ast::Node {
						id: 0,
						data: ast::NodeData::Sequence(vec![
							ast::Node {
								id: 0,
								data: ast::NodeData::FunctionCall(ast::FunctionCall {
									name: "print".to_string(),
									argv: vec![
										ast::Node { id: 0, data: ast::NodeData::RawText("8Hy".to_string()) },
										ast::Node {
											id: 0,
											data: ast::NodeData::FunctionCall(ast::FunctionCall {
												name: "print".to_string(),
												argv: vec![
													ast::Node { id: 0, data: ast::NodeData::RawText("9Hy".to_string()) },
													ast::Node { id: 0, data: ast::NodeData::RawText("0Mom".to_string()) },
												],
											}),
										},
									],
								}),
							},
							ast::Node {
								id: 0,
								data: ast::NodeData::FunctionCall(ast::FunctionCall {
									name: "print".to_string(),
									argv: vec![
										ast::Node { id: 0, data: ast::NodeData::RawText("45Hy".to_string()) },
										ast::Node { id: 0, data: ast::NodeData::RawText("56Mom".to_string()) },
									],
								}),
							},
							ast::Node { id: 0, data: ast::NodeData::RawText("My flex return".to_string()) },
							ast::Node {
								id: 0,
								data: ast::NodeData::FunctionCall(ast::FunctionCall {
									name: "print".to_string(),
									argv: vec![ast::Node { id: 0, data: ast::NodeData::RawText("last after".to_string()) }],
								}),
							},
						]),
					}],
				}),
			},
		]),
	}
}

fn main() {
	let a = ast_to_python(&b());
	print!("{a}");

	use std::io::Write;
	let mut file = std::fs::File::create("./runner/examples/example.py").unwrap();
	file.write_all(a.as_bytes()).unwrap();
}
