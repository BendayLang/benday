use models::{ast::*, change::*};

pub fn example_change() -> Vec<Change> {
	let ast: Node = Node { id: 0, data: crate::NodeData::Sequence(vec![]) };
	vec![
		Change { id_path: vec![0], data: ChangeData::Replace(ast.clone()) },
		Change { id_path: vec![0], data: ChangeData::Insert(Insert { inner_id_path: vec![0], ast_node: ast.clone() }) },
		Change { id_path: vec![0], data: ChangeData::Move(Move { inner_id_path: vec![0], new_parent_id_path: vec![0] }) },
		Change { id_path: vec![0], data: ChangeData::Delete },
	]
}

fn main() {
	use std::io::Write;
	let ast = example_change();
	let json = serde_json::to_string_pretty(&ast).unwrap();
	let mut file = std::fs::File::create("./models/examples/change.json").unwrap();
	file.write_all(json.as_bytes()).unwrap();
}
