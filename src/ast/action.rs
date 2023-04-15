use crate::ast::statement::Statement;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct Action {
    pub id: u32,
    pub statements: Vec<Statement>,
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "/*")?;
        writeln!(f, " * Action {}", self.id)?;
        writeln!(f, " */")?;
        writeln!(f)?;
        for statement in &self.statements {
            writeln!(f, "{}", statement)?
        }
        Ok(())
    }
}
