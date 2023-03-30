use std::fmt::{Display, Formatter};

pub mod binary_expr;
pub mod unary_expr;
pub mod expr;
pub mod variant;
pub mod statement;

#[derive(Debug, Clone)]
pub struct ASIdentifier {
    pub name: String,
}

impl Display for ASIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
