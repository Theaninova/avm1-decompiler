use crate::ast::expr::ASExpression;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub target: Box<ASExpression>,
    pub expression_type: UnaryExpressionType,
}

#[derive(Debug, Clone)]
pub enum UnaryExpressionType {
    Increment,
    Decrement,
    Not,
}

impl Display for UnaryExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.expression_type {
            UnaryExpressionType::Increment => write!(f, "({} + 1)", self.target),
            UnaryExpressionType::Decrement => write!(f, "({} - 1)", self.target),
            UnaryExpressionType::Not => write!(f, "!{}", self.target),
        }
    }
}
