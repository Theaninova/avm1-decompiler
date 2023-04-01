use crate::ast::expr::{Expression, ReferenceExpression};
use crate::ast::ASIdentifier;
use std::fmt::{Display, Formatter};
use swf::avm1::types::FunctionFlags;

#[derive(Debug, Clone)]
pub enum Statement {
    FunctionDeclaration(FunctionDeclaration),
    DefineLocal {
        left: ReferenceExpression,
        right: Expression,
    },
    SetVariable {
        left: ReferenceExpression,
        right: Expression,
    },
    SetMember {
        object: ReferenceExpression,
        name: ReferenceExpression,
        value: Expression,
    },
    StoreRegister {
        id: u8,
        value: Expression,
    },
    Return(Option<Expression>),
    UnknownStatement(String),
    ExpressionStatement(Expression),
    DanglingStack(Expression),
    Pop(Expression),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::FunctionDeclaration(x) => writeln!(f, "{}", x),
            Statement::DefineLocal { left, right } => writeln!(f, "var {} = {}", left, right),
            Statement::SetVariable { left, right } => writeln!(f, "{} = {}", left, right),
            Statement::SetMember {
                object,
                name,
                value,
            } => match &name {
                ReferenceExpression::Identifier(identifier) => {
                    writeln!(f, "{}.{} = {}", object, identifier, value)
                }
                ReferenceExpression::Variable(var) => {
                    writeln!(f, "{}[{}] = {}", object, var, value)
                }
                ReferenceExpression::Register(reg) => {
                    writeln!(f, "{}[${}] = {}", object, reg, value)
                }
                ReferenceExpression::Expression(expr) => {
                    writeln!(f, "{}[{}] = {}", object, expr, value)
                }
            },
            Statement::StoreRegister { id, value } => writeln!(f, "${} = {}", id, value),
            Statement::UnknownStatement(x) => writeln!(f, "??? {}", x),
            Statement::Return(value) => match value {
                Some(value) => writeln!(f, "return {}", value),
                None => writeln!(f, "return"),
            },
            Statement::ExpressionStatement(expression) => writeln!(f, "{}", expression),
            Statement::DanglingStack(stack) => {
                writeln!(f, "//!! {}", stack)
            }
            Statement::Pop(pop) => {
                writeln!(f, "// pop: {}", pop)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub identifier: Option<ASIdentifier>,
    pub flags: FunctionFlags,
    pub parameters: Vec<ReferenceExpression>,
    pub body: Vec<Statement>,
}

impl Display for FunctionDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.identifier {
            write!(f, "function {}(", name)?;
        } else {
            write!(f, "function (")?;
        }
        for param in self.parameters.iter() {
            write!(f, "{}", param)?
        }
        writeln!(f, ") {{")?;

        for stmt in self.body.iter() {
            write!(f, "  {}", stmt)?;
        }

        write!(f, "}}")
    }
}
