use models::{ast::Node, error::MathParsabilityError, return_value::ReturnValue};

#[derive(PartialEq)]
pub enum MathParsability {
	IntParsable,
	FloatParsable,
	Unparsable,
}

pub fn get_math_parsibility(expression: &str) -> MathParsability {
	let mut ns = fasteval::EmptyNamespace;

	match fasteval::ez_eval(expression, &mut ns) {
		Ok(v) => {
			if v.fract() == 0.0 {
				MathParsability::IntParsable
			} else {
				MathParsability::FloatParsable
			}
		}
		Err(_) => MathParsability::Unparsable,
	}
}

#[allow(unreachable_patterns)]
pub fn math_expression(expression: &str) -> Result<ReturnValue, MathParsabilityError> {
	let mut ns = fasteval::EmptyNamespace;
	match fasteval::ez_eval(expression, &mut ns) {
		Ok(v) => match v.fract() == 0.0 {
			true => Ok(ReturnValue::Int(v as isize)),
			false => Ok(ReturnValue::Float(v)),
		},
		Err(err) => match err {
			_ => Err(MathParsabilityError::IsNotMath),
			fasteval::Error::SlabOverflow => todo!(),
			fasteval::Error::AlreadyExists => todo!(),
			fasteval::Error::EOF => todo!(),
			fasteval::Error::EofWhileParsing(_) => todo!(),
			fasteval::Error::Utf8ErrorWhileParsing(_) => todo!(),
			fasteval::Error::TooLong => todo!(),
			fasteval::Error::TooDeep => todo!(),
			fasteval::Error::UnparsedTokensRemaining(_) => todo!(),
			fasteval::Error::InvalidValue => todo!(),
			fasteval::Error::ParseF64(_) => todo!(),
			fasteval::Error::Expected(_) => todo!(),
			fasteval::Error::WrongArgs(_) => todo!(),
			fasteval::Error::Undefined(_) => todo!(),
			fasteval::Error::Unreachable => todo!(),
		},
	}
}
