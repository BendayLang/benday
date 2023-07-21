use std::collections::HashMap;

pub type Id = u32;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Node {
	pub id: Id,
	#[serde(flatten)]
	pub data: NodeData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum NodeData {
	Sequence(Sequence),
	While(While),
	IfElse(IfElse),
	RawText(String), // could be rename to Literal
	VariableAssignment(VariableAssignment),
	FunctionCall(FunctionCall),
	FunctionDeclaration(FunctionDeclaration),
}

type Sequence = Vec<Node>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct While {
	pub is_do: bool,
	pub condition: Box<Node>,
	pub sequence: Box<Node>,
}

// TODO remove the if, since it's just a IfElse with no elif and no else (that are Optionnal anyway)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct If {
	pub condition: Box<Node>,
	pub sequence: Box<Node>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct IfElse {
	#[serde(rename = "if")]
	pub if_: If,
	pub elif: Option<Vec<If>>,
	#[serde(rename = "else")]
	pub else_: Option<Box<Node>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VariableAssignment {
	pub name: String,
	pub value: Box<Node>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCall {
	pub name: String, // TODO: un id ?
	pub argv: Vec<Node>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FunctionDeclaration {
	pub name: String,
	pub argv: HashMap<String, VariableAssignment>,
	pub sequence: Box<Node>,
}
