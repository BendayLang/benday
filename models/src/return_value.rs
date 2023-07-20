use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum ReturnValue {
	#[serde(rename = "string")]
	r#String(String),
	Int(isize),
	Float(f64),
	Bool(bool),
}

impl fmt::Display for ReturnValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ReturnValue::String(str) => write!(f, "{str}"),
			ReturnValue::Int(val) => write!(f, "{val}"),
			ReturnValue::Float(val) => write!(f, "{val}"),
			ReturnValue::Bool(val) => write!(f, "{val}"),
		}
	}
}
