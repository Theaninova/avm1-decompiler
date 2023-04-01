use std::fmt::{Display, Formatter};

pub mod binary_expr;
pub mod expr;
pub mod statement;
pub mod variant;

#[derive(Debug, Clone)]
pub struct ASIdentifier {
    pub name: String,
}

impl Display for ASIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
