use crate::ast::expr::Expression;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum Variant {
    Uninitialized,
    Undefined,
    Null,
    Bool(bool),
    Int(i32),
    Float(f32),
    Double(f64),
    String(String),
    Array(Vec<Expression>),
    Object(Vec<(Expression, Expression)>),
}

impl Display for Variant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::Uninitialized => write!(f, "!!!"),
            Variant::Undefined => write!(f, "undefined"),
            Variant::Null => write!(f, "null"),
            Variant::Bool(value) => write!(f, "{}", value),
            Variant::Int(value) => write!(f, "{}", value),
            Variant::Float(value) => write!(f, "{:?}", value),
            Variant::Double(value) => write!(f, "{:?}", value),
            Variant::String(value) => write!(f, "\"{}\"", value),
            Variant::Array(value) => {
                let members_fmt: Vec<String> = value.iter().map(|x| x.to_string()).collect();
                write!(f, "[{}]", members_fmt.join(", "))
            }
            Variant::Object(value) => {
                let members_fmt: Vec<String> =
                    value.iter().map(|(a, b)| format!("{}: {}", a, b)).collect();
                write!(f, "{{{}}}", members_fmt.join(", "))
            }
        }
    }
}
