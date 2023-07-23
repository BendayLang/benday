use models::{
	self,
	ast::{FunctionCall, FunctionDeclaration, IfElse, Node, NodeData, VariableAssignment, While},
};
use python_parser::{
	ast::{self, Expression},
	visitors::printer::format_module,
};
use rand;
use std::{
	collections::{HashSet, VecDeque},
	fmt::format,
};
use wtf8;

type DoesReturn = bool;

#[derive(Default)]
pub struct UidGenerator(HashSet<u32>);

impl UidGenerator {
	fn new_id(&mut self) -> u32 {
		let rnd = rand::random::<u32>();
		if self.0.contains(&rnd) {
			self.new_id()
		} else {
			self.0.insert(rnd);
			rnd
		}
	}

	fn new_id_str(&mut self) -> String {
		format!("{:010}", self.new_id())
	}

	pub fn gen_lambda_sequence_name(&mut self) -> String {
		format!("lambda_sequence_{}", self.new_id_str())
	}

	pub fn gen_arg_name(&mut self, nth: usize, fn_name: &str) -> String {
		format!("arg_{nth}_{}_{fn_name}", self.new_id_str())
	}
}

macro_rules! py_str {
	($string:ident) => {
		python_parser::ast::PyString { prefix: String::new(), content: wtf8::Wtf8Buf::from_str($string) }
	};
	($prefix:expr, $string:ident) => {
		python_parser::ast::PyString { prefix: $prefix.to_string(), content: wtf8::Wtf8Buf::from_str($string) }
	};
}

/// (Identifier, Optionnal body, does return)
fn run(ast: &Node, is_first: bool, uid_generator: &mut UidGenerator) -> Vec<ast::Statement> {
	match &ast.data {
		NodeData::RawText(string) => {
			vec![ast::Statement::Assignment(
				vec![ast::Expression::Name("e".to_string())],
				vec![vec![ast::Expression::String(vec![py_str!(string)])]],
			)]
		}
		NodeData::FunctionCall(FunctionCall { name, argv }) => {
			vec![ast::Statement::Assignment(
				vec![ast::Expression::Call(Box::new(ast::Expression::Name(name.clone())), vec![])],
				vec![],
			)]
		}
		_ => todo!(),
		// 	NodeData::FunctionCall(FunctionCall { name, argv }) => {
		// 		let mut sub_identifiers: Vec<String> = Vec::new();
		// 		let mut return_body = VecDeque::new();
		// 		for arg in argv.iter() {
		// 			let (identifier, body, does_return) = run(arg, false, uid_generator);
		// 			sub_identifiers.push(identifier);
		// 			if let Some(mut body) = body {
		// 				return_body.append(&mut body);
		// 			}
		// 		}
		// 		let return_body = if return_body.is_empty() { None } else { Some(return_body) };
		// 		let mut return_identifier = name.to_string() + "(";
		// 		for i in &sub_identifiers {
		// 			return_identifier += &i;
		// 			if &i != &sub_identifiers.last().unwrap() {
		// 				return_identifier += ", ";
		// 			}
		// 		}
		// 		return_identifier += ")";
		// 		(return_identifier, return_body, false) // TODO -> en fonction de chaque fn ?
		// 	}
		// 	NodeData::Sequence(nodes) => {
		// 		let sequence_name = if is_first { "main".to_string() } else { uid_generator.gen_lambda_sequence_name() };
		// 		let mut return_body = VecDeque::new();
		// 		let mut final_does_return = false;
		// 		for node in nodes.iter() {
		// 			let (identifier, body, does_return) = run(node, false, uid_generator);
		// 			if let Some(mut body) = body {
		// 				return_body.append(&mut body.iter().map(|line| format!("\t{line}")).collect::<VecDeque<String>>());
		// 			}
		// 			if does_return {
		// 				return_body.push_back(format!("\treturn {identifier}"));
		// 				final_does_return = true;
		// 				break;
		// 			} else {
		// 				return_body.push_back(format!("\t{identifier}"));
		// 			}
		// 		}
		// 		let return_body = if return_body.is_empty() {
		// 			None
		// 		} else {
		// 			return_body.push_front(format!("def {sequence_name}():"));
		// 			Some(return_body)
		// 		};
		// 		(format!("{sequence_name}()"), return_body, final_does_return)
		// 	}
		// 	NodeData::VariableAssignment(VariableAssignment { name, value }) => {
		// 		let (identifier, body, does_return) = run(&value, false, uid_generator);
		// 		(format!("{name} = {identifier}"), body, false)
		// 	}
		// 	NodeData::While(While { is_do, condition, sequence }) => {
		// 		if *is_do {
		// 			todo!("no do: loop in python");
		// 		}
		// 		todo!()
		// 	}
		// 	NodeData::IfElse(IfElse { r#if, elif, r#else }) => todo!(),
		// 	NodeData::FunctionDeclaration(FunctionDeclaration { name, argv, sequence }) => todo!(),
	}
}

pub fn ast_to_python(ast: &Node) -> String {
	let mut uid_generator = UidGenerator::default();
	let ast = run(ast, true, &mut uid_generator);
	format_module(&ast)
}
