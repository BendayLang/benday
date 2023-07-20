use models::error::*;

// pub enum ErrorType {
//     InfiniteLoop {
//         reaches: usize,
//         max: usize,
//     },
//     VariableExpansionError(VariableExpansionError),
//     RootIsNotSequence,
//     ConditionAlwaysFalse,
//     ConditionAlwaysTrue,
//     InvalidType {
//         accepted: Vec<ReturnValue>,
//         found: ReturnValue,
//     },
// }
pub fn error_example() -> Vec<ErrorMessage> {
	vec![
		ErrorMessage::new(vec![0], ErrorType::InfiniteLoop { reaches: 546, max: 500 }, None),
		ErrorMessage::new(vec![0], ErrorType::InfiniteLoop { reaches: 546, max: 500 }, Some("test")),
		ErrorMessage::new(vec![0], ErrorType::VariableExpansionError(VariableExpansionError::VariableNotExpandable), None),
		ErrorMessage::new(vec![0], ErrorType::VariableExpansionError(VariableExpansionError::MissingClosingBracket), None),
		ErrorMessage::new(vec![0], ErrorType::VariableExpansionError(VariableExpansionError::MissingOpeningBracket), None),
		ErrorMessage::new(vec![0], ErrorType::VariableExpansionError(VariableExpansionError::BracketOrder), None),
		ErrorMessage::new(
			vec![0],
			ErrorType::VariableExpansionError(VariableExpansionError::VariableNotFound("test".to_string())),
			None,
		),
		ErrorMessage::new(vec![0], ErrorType::RootIsNotSequence, None),
		ErrorMessage::new(vec![0], ErrorType::ConditionAlwaysFalse, None),
		ErrorMessage::new(vec![0], ErrorType::ConditionAlwaysTrue, None),
		ErrorMessage::new(vec![0], ErrorType::InvalidType { accepted: vec![ValueType::String_], found: ValueType::Float }, None),
	]
}

fn main() {
	use std::io::Write;
	let ast = error_example();
	let json = serde_json::to_string(&ast).unwrap();
	let mut file = std::fs::File::create("./models/examples/error.json").unwrap();
	file.write_all(json.as_bytes()).unwrap();
}
