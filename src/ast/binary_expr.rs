use crate::ast::expr::ASExpression;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<ASExpression>,
    pub right: Box<ASExpression>,
    pub expression_type: BinaryExpressionType,
}

#[derive(Debug, Clone)]
pub enum BinaryExpressionType {
    Add,
    Subtract,
    Divide,
    Multiply,
    Modulo,
    Equals,
    StrictEquals,
    LogicalAnd,
    LogicalOr,
    Less,
    Greater,
    BitAnd,
    BitLShift,
    BitOr,
    BitRShift,
    BitURShift,
    BitXor,
}

impl Display for BinaryExpressionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinaryExpressionType::Less => "<",
                BinaryExpressionType::Greater => ">",
                BinaryExpressionType::Add => "+",
                BinaryExpressionType::BitOr => "|",
                BinaryExpressionType::BitAnd => "&",
                BinaryExpressionType::BitLShift => "<<",
                BinaryExpressionType::BitRShift => ">>",
                BinaryExpressionType::BitURShift => ">>>",
                BinaryExpressionType::BitXor => "^",
                BinaryExpressionType::Divide => "/",
                BinaryExpressionType::Multiply => "*",
                BinaryExpressionType::Modulo => "%",
                BinaryExpressionType::Subtract => "-",
                BinaryExpressionType::Equals => "==",
                BinaryExpressionType::LogicalAnd => "&&",
                BinaryExpressionType::LogicalOr => "||",
                BinaryExpressionType::StrictEquals => "===",
            }
        )
    }
}
