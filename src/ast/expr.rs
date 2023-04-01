use crate::ast::binary_expr::BinaryExpressionType;
use crate::ast::statement::FunctionDeclaration;
use crate::ast::variant::Variant;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum Expression {
    Reference(ReferenceExpression),
    Function(FunctionDeclaration),
    GetMember {
        object: ReferenceExpression,
        name: ReferenceExpression,
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
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Reference(reference) => write!(f, "{}", reference),
            Expression::GetMember { object, name } => match name.clone() {
                ReferenceExpression::Identifier(identifier) => {
                    write!(f, "{}.{}", object, identifier)
                }
                ReferenceExpression::Variable(var) => {
                    write!(f, "{}[{}]", name, var)
                }
                ReferenceExpression::Register(reg) => write!(f, "{}[${}]", object, reg),
                ReferenceExpression::Expression(expr) => write!(f, "{}[{}]", object, expr),
            },
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
            Expression::Function(function) => write!(f, "{}", function),
            Expression::Unary {
                target,
                expression_type,
            } => match expression_type {
                UnaryExpressionType::Increment => write!(f, "({} + 1)", target),
                UnaryExpressionType::Decrement => write!(f, "({} - 1)", target),
                UnaryExpressionType::Not => write!(f, "!{}", target),
                UnaryExpressionType::ToInteger => write!(f, "int({})", target),
                UnaryExpressionType::ToString => write!(f, "String({})", target),
                UnaryExpressionType::ToNumber => writeln!(f, "Number({})", target),
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
