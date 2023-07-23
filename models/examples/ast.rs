use models::ast::*;
use std::collections::HashMap;

pub fn ast_example() -> Node {
	Node {
		id: 0,
		data: NodeData::Sequence(vec![
			Node {
				id: 1,
				data: NodeData::VariableAssignment(VariableAssignment {
					name: "age de Bob".to_string(),
					value: Box::new(Node { id: 2, data: NodeData::RawText("6".to_string()) }),
				}),
			},
			Node {
				id: 201,
				data: NodeData::IfElse(IfElse {
					r#if: If {
						condition: Box::new(Node { id: 202, data: NodeData::RawText("1".to_string()) }),
						sequence: Box::new(Node { id: 203, data: NodeData::Sequence(vec![]) }),
					},
					elif: None,
					r#else: None,
				}),
			},
			Node {
				id: 3,
				data: NodeData::While(While {
					is_do: false,
					condition: Box::new(Node { id: 4, data: NodeData::RawText("{age de Bob} < 13".to_string()) }),
					sequence: Box::new(Node {
						id: 200,
						data: NodeData::Sequence(vec![
							Node {
								id: 5,
								data: NodeData::VariableAssignment(VariableAssignment {
									name: "age de Bob".to_string(),
									value: Box::new(Node { id: 6, data: NodeData::RawText("{age de Bob} + 1".to_string()) }),
								}),
							},
							Node {
								id: 7,
								data: NodeData::FunctionCall(FunctionCall {
									name: "print".to_string(),
									argv: vec![Node {
										id: 8,
										data: NodeData::RawText(
											"Bravo Bob ! tu as maintenant \"{age de Bob}\" ans !".to_string(),
										),
									}],
								}),
							},
						]),
					}),
				}),
			},
			Node {
				id: 9,
				data: NodeData::FunctionCall(FunctionCall {
					name: "print".to_string(),
					argv: vec![Node { id: 10, data: NodeData::RawText("Bob est parti a l'age de {age de Bob} !".to_string()) }],
				}),
			},
			Node {
				id: 11,
				data: NodeData::FunctionDeclaration(FunctionDeclaration {
					name: "print text and number".to_string(),
					sequence: Box::new(Node {
						id: 201,
						data: NodeData::Sequence(vec![
							Node {
								id: 14,
								data: NodeData::FunctionCall(FunctionCall {
									name: "print".to_string(),
									argv: vec![Node { id: 15, data: NodeData::RawText("{text}".to_string()) }],
								}),
							},
							Node {
								id: 16,
								data: NodeData::FunctionCall(FunctionCall {
									name: "print".to_string(),
									argv: vec![Node { id: 17, data: NodeData::RawText("{number}".to_string()) }],
								}),
							},
						]),
					}),

					argv: HashMap::from([
						(
							"text".to_string(),
							VariableAssignment {
								name: "text".to_string(),
								value: Box::new(Node { id: 12, data: NodeData::RawText("".to_string()) }),
							},
						),
						(
							"number".to_string(),
							VariableAssignment {
								name: "number".to_string(),
								value: Box::new(Node { id: 13, data: NodeData::RawText("".to_string()) }),
							},
						),
					]),
				}),
			},
		]),
	}
}

fn main() {
	use std::io::Write;
	let ast = ast_example();
	let json = serde_json::to_string_pretty(&ast).unwrap();
	let mut file = std::fs::File::create("./models/examples/ast.json").unwrap();
	file.write_all(json.as_bytes()).unwrap();
}
