use crate::ast::binary_expr::BinaryExpression;
use crate::ast::unary_expr::UnaryExpression;
use crate::ast::variant::Variant;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum ASExpression {
    Reference(ASReferenceExpression),
    GetMember(ASGetMemberExpression),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Literal(Variant),
    CallFunction(ASFunctionCallExpression),
}

impl Display for ASExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ASExpression::Reference(reference) => write!(f, "{}", reference),
            ASExpression::GetMember(member) => match *member.name.clone() {
                ASReferenceExpression::Identifier(identifier) => {
                    write!(f, "{}.{}", member.object, identifier)
                }
                ASReferenceExpression::Register(reg) => write!(f, "{}[${}]", member.object, reg),
                ASReferenceExpression::Expression(expr) => write!(f, "{}[{}]", member.object, expr),
            },
            ASExpression::Literal(literal) => write!(f, "{}", literal),
            ASExpression::Binary(binary) => write!(
                f,
                "({} {} {})",
                binary.left, binary.expression_type, binary.right
            ),
            ASExpression::Unary(unary) => write!(f, "{}", unary),
            ASExpression::CallFunction(call) => {
                let args: Vec<String> = call.args.iter().map(|it| it.to_string()).collect();
                write!(f, "{}({})", call.name, args.join(", "))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ASFunctionCallExpression {
    pub name: ASReferenceExpression,
    pub args: Vec<ASExpression>,
}

#[derive(Debug, Clone)]
pub struct ASGetMemberExpression {
    pub name: Box<ASReferenceExpression>,
    pub object: Box<ASReferenceExpression>,
}

#[derive(Debug, Clone)]
pub enum ASReferenceExpression {
    Identifier(String),
    Register(u8),
    Expression(Box<ASExpression>),
}

impl ASReferenceExpression {
    pub fn from_expression(expr: ASExpression) -> ASReferenceExpression {
        match expr {
            ASExpression::Literal(Variant::String(string)) => {
                ASReferenceExpression::Identifier(string)
            }
            ASExpression::Reference(reference) => reference,
            _ => ASReferenceExpression::Expression(Box::new(expr)),
        }
    }
}

impl Display for ASReferenceExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ASReferenceExpression::Identifier(it) => write!(f, "{}", it),
            ASReferenceExpression::Expression(expr) => write!(f, "{}", expr),
            ASReferenceExpression::Register(reg) => write!(f, "${}", reg),
        }
    }
}
