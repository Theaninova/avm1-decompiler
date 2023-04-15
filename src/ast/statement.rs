use crate::ast::block::Block;
use crate::ast::expr::{Expression, ReferenceExpression};
use std::fmt::{Display, Formatter};


#[derive(Debug, Clone)]
pub enum Statement {
    DefineLocal {
        left: ReferenceExpression,
        right: Expression,
    },
    DeclareLocal {
        name: ReferenceExpression,
    },
    SetMember {
        object: ReferenceExpression,
        name: ReferenceExpression,
        value: Expression,
    },
    
    If {
        condition: Expression,
        true_branch: Block,
        false_branch: Block,
    },
    Trace(Expression),
    Return(Option<Expression>),
    UnknownStatement(String),
    ExpressionStatement(Expression),
    DanglingStack(Expression),
    GotoLabel(String),
    GotoFrame(u16),
    Play,
    Stop,
    Pop(Expression),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::DefineLocal { left, right } => write!(f, "var {} = {}", left, right),
            Statement::DeclareLocal { name } => write!(f, "var {}", name),
            Statement::Trace(expr) => write!(f, "trace({})", expr),
            Statement::Play => write!(f, "play()"),
            Statement::Stop => write!(f, "stop()"),
            Statement::GotoLabel(label) => write!(f, "gotoAndPlay({})", label),
            Statement::GotoFrame(frame) => write!(f, "gotoAndPlay({})", frame),
            Statement::If {
                condition,
                true_branch,
                false_branch,
            } => write!(
                f,
                "if ({}) {} else {}",
                condition, true_branch, false_branch
            ),
            Statement::SetMember {
                object,
                name,
                value,
            } => match &name {
                ReferenceExpression::Identifier(identifier) => {
                    write!(f, "{}.{} = {}", object, identifier, value)
                }
                ReferenceExpression::Variable(var) => {
                    write!(f, "{}[{}] = {}", object, var, value)
                }
                ReferenceExpression::Register(reg) => {
                    write!(f, "{}[${}] = {}", object, reg, value)
                }
                ReferenceExpression::Expression(expr) => {
                    write!(f, "{}[{}] = {}", object, expr, value)
                }
            },
            Statement::UnknownStatement(x) => write!(f, "??? {}", x),
            Statement::Return(value) => match value {
                Some(value) => write!(f, "return {}", value),
                None => write!(f, "return"),
            },
            Statement::ExpressionStatement(expression) => write!(f, "{}", expression),
            Statement::DanglingStack(stack) => {
                write!(f, "{}", stack)
            }
            Statement::Pop(pop) => {
                write!(f, "// pop: {}", pop)
            }
        }
    }
}
