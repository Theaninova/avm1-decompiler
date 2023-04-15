use crate::ast::statement::Statement;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct Block {
    pub body: Vec<Statement>,
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for statement in &self.body {
            writeln!(f, "{}", indent(statement.to_string()))?;
        }
        writeln!(f, "}}")
    }
}

fn indent(str: String) -> String {
    str.split('\n')
        .into_iter()
        .map(|x| format!("  {}", x))
        .join("\n")
}
