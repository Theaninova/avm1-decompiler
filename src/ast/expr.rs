use crate::ast::binary_expr::BinaryExpression;
use crate::ast::statement::FunctionDeclaration;
use crate::ast::unary_expr::UnaryExpression;
use crate::ast::variant::Variant;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum Expression {
    Reference(ReferenceExpression),
    Function(FunctionDeclaration),
    GetMember(ASGetMemberExpression),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Literal(Variant),
    CallFunction(ASFunctionCallExpression),
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Reference(reference) => write!(f, "{}", reference),
            Expression::GetMember(member) => match *member.name.clone() {
                ReferenceExpression::Identifier(identifier) => {
                    write!(f, "{}.{}", member.object, identifier)
                }
                ReferenceExpression::Register(reg) => write!(f, "{}[${}]", member.object, reg),
                ReferenceExpression::Expression(expr) => write!(f, "{}[{}]", member.object, expr),
            },
            Expression::Literal(literal) => write!(f, "{}", literal),
            Expression::Binary(binary) => write!(
                f,
                "({} {} {})",
                binary.left, binary.expression_type, binary.right
            ),
            Expression::Function(function) => write!(f, "{}", function),
            Expression::Unary(unary) => write!(f, "{}", unary),
            Expression::CallFunction(call) => {
                let args: Vec<String> = call.args.iter().map(|it| it.to_string()).collect();
                write!(f, "{}({})", call.name, args.join(", "))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ASFunctionCallExpression {
    pub name: ReferenceExpression,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct ASGetMemberExpression {
    pub name: Box<ReferenceExpression>,
    pub object: Box<ReferenceExpression>,
}

#[derive(Debug, Clone)]
pub enum ReferenceExpression {
    Identifier(String),
    Register(u8),
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
            ReferenceExpression::Expression(expr) => write!(f, "{}", expr),
            ReferenceExpression::Register(reg) => write!(f, "${}", reg),
        }
    }
}
