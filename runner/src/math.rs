use models::{ast::Node, return_value::ReturnValue};

#[derive(PartialEq)]
pub enum MathParsability {
	IntParsable,
	FloatParsable,
	Unparsable,
}

pub fn get_math_parsibility(expression: &str) -> MathParsability {
	let mut ns = fasteval::EmptyNamespace;

	return match fasteval::ez_eval(expression, &mut ns) {
		Ok(v) => {
			if v.fract() == 0.0 {
				MathParsability::IntParsable
			} else {
				MathParsability::FloatParsable
			}
		}
		Err(_) => MathParsability::Unparsable,
	};
}

pub fn math_expression(expression: &str) -> Result<ReturnValue, ()> {
	let mut ns = fasteval::EmptyNamespace;
	return match fasteval::ez_eval(expression, &mut ns) {
		Ok(v) => match v.fract() == 0.0 {
			true => Ok(ReturnValue::Int(v as isize)),
			false => Ok(ReturnValue::Float(v)),
		},
		Err(_) => Err(()),
	};
}
