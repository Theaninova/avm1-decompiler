use crate::ast::expr::{Expression, ReferenceExpression};
use crate::ast::ASIdentifier;
use std::fmt::{Display, Formatter};
use swf::avm1::types::FunctionFlags;

#[derive(Debug, Clone)]
pub enum Statement {
    FunctionDeclaration(FunctionDeclaration),
    DefineLocal(DefineLocal),
    SetVariable(SetVariable),
    SetMember(SetMember),
    StoreRegister(StoreRegister),
    Return(Option<Expression>),
    UnknownStatement(String),
    ExpressionStatement(Expression),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::FunctionDeclaration(x) => writeln!(f, "{}", x),
            Statement::DefineLocal(x) => writeln!(f, "var {} = {}", x.left, x.right),
            Statement::SetVariable(x) => writeln!(f, "{} = {}", x.left, x.right),
            Statement::SetMember(x) => match &x.name {
                ReferenceExpression::Identifier(identifier) => {
                    writeln!(f, "{}.{} = {}", x.object, identifier, x.value)
                }
                ReferenceExpression::Register(reg) => {
                    writeln!(f, "{}[${}] = {}", x.object, reg, x.value)
                }
                ReferenceExpression::Expression(expr) => {
                    writeln!(f, "{}[{}] = {}", x.object, expr, x.value)
                }
            },
            Statement::StoreRegister(x) => writeln!(f, "${} = {}", x.id, x.value),
            Statement::UnknownStatement(x) => writeln!(f, "??? {}", x),
            Statement::Return(value) => match value {
                Some(value) => writeln!(f, "return {}", value),
                None => writeln!(f, "return"),
            },
            Statement::ExpressionStatement(expression) => writeln!(f, "{}", expression),
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

#[derive(Debug, Clone)]
pub struct SetVariable {
    pub left: ReferenceExpression,
    pub right: Expression,
}

#[derive(Debug, Clone)]
pub struct DefineLocal {
    pub left: ReferenceExpression,
    pub right: Expression,
}

#[derive(Debug, Clone)]
pub struct SetMember {
    pub value: Expression,
    pub name: ReferenceExpression,
    pub object: ReferenceExpression,
}

#[derive(Debug, Clone)]
pub struct StoreRegister {
    pub id: u8,
    pub value: Expression,
}
