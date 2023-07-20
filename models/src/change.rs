use serde::{Deserialize, Serialize};

use crate::ast::Node;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
	pub id_path: Vec<usize>,
	#[serde(flatten)]
	pub data: ChangeData,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum ChangeData {
	Replace(Node),
	Delete,
	Move(Move),
	Insert(Insert),
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Insert {
	pub inner_id_path: Vec<usize>, // TODO quel est le meilleur format ?
	pub ast_node: Node,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Move {
	pub new_parent_id_path: Vec<usize>,
	pub inner_id_path: Vec<usize>, // TODO quel est le meilleur format ?
}

// TODO -> specify the inner path of the node
// pub enum InnerPath {
//     /// Left or right = name or value
//     VariableAssignment(either::Either<(), ()>),
//     /// Left = name, right = value(index)
//     FunctionCall(either::Either<(), usize>),

//     While...
// }
