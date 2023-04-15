use crate::ast::binary_expr::BinaryExpressionType;
use crate::ast::block::Block;
use crate::ast::variant::Variant;
use itertools::Itertools;
use std::fmt::{Display, Formatter};
use swf::avm1::types::FunctionFlags;

#[derive(Debug, Clone)]
pub struct SuperpositionExpression {
    pub id: Option<usize>,
    pub value: Box<Expression>,
}

impl Display for SuperpositionExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(cond) = &self.id {
            write!(f, "{}|{}⟩", cond, self.value)
        } else {
            write!(f, "|{}⟩", self.value)
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Superposition(Vec<SuperpositionExpression>),
    Reference(ReferenceExpression),
    Function {
        identifier: Option<String>,
        flags: FunctionFlags,
        parameters: Vec<ReferenceExpression>,
        body: Block,
    },
    GetMember {
        object: ReferenceExpression,
        name: ReferenceExpression,
    },
    GetProperty {
        object: ReferenceExpression,
        name: ReferenceExpression,
    },
    Ternary {
        condition: Box<Expression>,
        if_true: Box<Expression>,
        if_false: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        right: Box<Expression>,
        expression_type: BinaryExpressionType,
    },
    Unary {
        target: Box<Expression>,
        expression_type: UnaryExpressionType,
    },
    Literal(Variant),
    CallFunction {
        name: ReferenceExpression,
        args: Vec<Expression>,
    },
    CallMethod {
        object: ReferenceExpression,
        name: ReferenceExpression,
        args: Vec<Expression>,
    },
    StoreRegister {
        id: u8,
        value: Box<Expression>,
    },
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Ternary {
                condition,
                if_false,
                if_true,
            } => write!(f, "{} ? {} : {}", condition, if_true, if_false),
            Expression::StoreRegister { id, value } => write!(f, "${} = {}", id, value),
            Expression::Superposition(superpositions) => {
                write!(
                    f,
                    "{}",
                    superpositions.iter().map(|it| it.to_string()).join(" + ")
                )
            }
            Expression::Reference(reference) => write!(f, "{}", reference),
            Expression::GetMember { object, name } | Expression::GetProperty { object, name } => {
                match name.clone() {
                    ReferenceExpression::Identifier(identifier) => {
                        write!(f, "{}.{}", object, identifier)
                    }
                    ReferenceExpression::Variable(var) => {
                        write!(f, "{}[{}]", name, var)
                    }
                    ReferenceExpression::Register(reg) => write!(f, "{}[${}]", object, reg),
                    ReferenceExpression::Expression(expr) => write!(f, "{}[{}]", object, expr),
                }
            }
            Expression::CallMethod { object, name, args } => {
                let args: Vec<String> = args.iter().map(|it| it.to_string()).collect();
                match name.clone() {
                    ReferenceExpression::Identifier(identifier) => {
                        write!(f, "{}.{}({})", object, identifier, args.join(", "))
                    }
                    ReferenceExpression::Variable(var) => {
                        write!(f, "{}[{}]({})", name, var, args.join(", "))
                    }
                    ReferenceExpression::Register(reg) => {
                        write!(f, "{}[${}]({})", object, reg, args.join(", "))
                    }
                    ReferenceExpression::Expression(expr) => {
                        write!(f, "{}[{}]({})", object, expr, args.join(", "))
                    }
                }
            }
            Expression::Literal(literal) => write!(f, "{}", literal),
            Expression::Binary {
                left,
                right,
                expression_type,
            } => write!(f, "({} {} {})", left, expression_type, right),
            Expression::Function {
                identifier,
                parameters,
                body,
                ..
            } => write!(
                f,
                "function {}({}) {}",
                if let Some(name) = identifier {
                    name
                } else {
                    ""
                },
                parameters.iter().map(|p| p.to_string()).join(", "),
                body
            ),
            Expression::Unary {
                target,
                expression_type,
            } => match expression_type {
                UnaryExpressionType::Increment => write!(f, "{}++", target),
                UnaryExpressionType::Decrement => write!(f, "{}--", target),
                UnaryExpressionType::Not => write!(f, "!{}", target),
                UnaryExpressionType::ToInteger => write!(f, "int({})", target),
                UnaryExpressionType::ToString => write!(f, "String({})", target),
                UnaryExpressionType::ToNumber => write!(f, "Number({})", target),
            },
            Expression::CallFunction { name, args } => {
                let args: Vec<String> = args.iter().map(|it| it.to_string()).collect();
                write!(f, "{}({})", name, args.join(", "))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaryExpressionType {
    Increment,
    Decrement,
    Not,
    ToInteger,
    ToString,
    ToNumber,
}

#[derive(Debug, Clone)]
pub enum ReferenceExpression {
    Identifier(String),
    Register(u8),
    Variable(String),
    Expression(Box<Expression>),
}

impl ReferenceExpression {
    pub fn from_expression(expr: Expression) -> ReferenceExpression {
        match expr {
            Expression::Literal(Variant::String(string)) => ReferenceExpression::Identifier(string),
            Expression::Reference(reference) => reference,
            _ => ReferenceExpression::Expression(Box::new(expr)),
        }
    }
}

impl Display for ReferenceExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferenceExpression::Identifier(it) => write!(f, "{}", it),
            ReferenceExpression::Variable(var) => write!(f, "{}", var),
            ReferenceExpression::Expression(expr) => write!(f, "{}", expr),
            ReferenceExpression::Register(reg) => write!(f, "${}", reg),
        }
    }
}
