use crate::{ast::Id, error::ErrorMessage, return_value::ReturnValue};
use std::collections::HashMap;

pub type AstResult = Result<Option<ReturnValue>, Vec<ErrorMessage>>;
pub type IdPath = Vec<Id>;

// La cle contiens le nom de la variable et l'id de le sequence (scope) dans laquelle elle a ete declaree
// Ca va permettre de gerer les variables locales et globales (recursivement)
pub type VariableMap = HashMap<(String, Id), ReturnValue>;
