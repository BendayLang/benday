use models::{ast::Id, error::ErrorType, return_value::ReturnValue, runner::AstResult};

#[derive(Debug, PartialEq)]
pub struct Action {
	r#type: ActionType,
	state_index: usize,
	node_id: Id,
}

impl Action {
	pub fn new(r#type: ActionType, state_index: usize, node_id: Id) -> Self {
		Self { r#type, state_index, node_id }
	}
	
	pub fn get_type(&self) -> &ActionType {
		&self.r#type
	}
}

#[derive(Debug, PartialEq)]
pub enum ActionType {
	Return(AstResult),
	CheckVarNameValidity(Result<(), ErrorType>),
	EvaluateRawText,
	AssignVariable { key: (String, Id), value: ReturnValue },
	CallBuildInFn(String),
	PushStdout(String),
	GetArgs,
	ControlFlowEvaluateCondition,
	Error(ErrorType),
}

impl ActionType {
	pub fn return_some(return_value: ReturnValue) -> Self {
		Self::Return(Ok(Some(return_value)))
	}
}
