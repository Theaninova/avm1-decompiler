use crate::ast::binary_expr::BinaryExpressionType;
use crate::ast::expr::{Expression, UnaryExpressionType};
use crate::decompiler::vm::VirtualMachine;
use swf::error::Result;

pub fn decompile_unary_expr(
    vm: &mut VirtualMachine,
    expression_type: UnaryExpressionType,
) -> Result<()> {
    let target = vm.pop()?;
    let is_negate = matches!(&expression_type, UnaryExpressionType::Not);
    vm.push(match target {
        Expression::Unary {
            target,
            expression_type: UnaryExpressionType::Not,
        } if is_negate => *target,
        Expression::Binary {
            left,
            right,
            expression_type: BinaryExpressionType::Equals,
        } if is_negate => Expression::Binary {
            left,
            right,
            expression_type: BinaryExpressionType::NotEquals,
        },
        Expression::Binary {
            left,
            right,
            expression_type: BinaryExpressionType::StrictEquals,
        } if is_negate => Expression::Binary {
            left,
            right,
            expression_type: BinaryExpressionType::NotStrictEquals,
        },
        Expression::Binary {
            left,
            right,
            expression_type: BinaryExpressionType::NotEquals,
        } if is_negate => Expression::Binary {
            left,
            right,
            expression_type: BinaryExpressionType::Equals,
        },
        Expression::Binary {
            left,
            right,
            expression_type: BinaryExpressionType::NotStrictEquals,
        } if is_negate => Expression::Binary {
            left,
            right,
            expression_type: BinaryExpressionType::StrictEquals,
        },
        _ => Expression::Unary {
            target: Box::new(target),
            expression_type,
        },
    });
    Ok(())
}
