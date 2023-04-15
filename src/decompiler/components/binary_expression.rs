use crate::ast::binary_expr::BinaryExpressionType;
use crate::ast::expr::Expression;
use crate::decompiler::vm::VirtualMachine;
use swf::error::Result;

pub fn decompile_binary_expr(
    vm: &mut VirtualMachine,
    expression_type: BinaryExpressionType,
) -> Result<()> {
    let right = vm.pop()?;
    let left = vm.pop()?;
    vm.push(Expression::Binary {
        left: Box::new(left),
        right: Box::new(right),
        expression_type,
    });
    Ok(())
}
