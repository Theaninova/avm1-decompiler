use crate::ast::block::Block;
use crate::ast::expr::Expression;
use crate::ast::statement::Statement;
use crate::decompiler::vm::VirtualMachine;
use itertools::Itertools;
use swf::error::{Error, Result};

pub fn resolve_loop(vm: &mut VirtualMachine, target: usize) -> Result<()> {
    let (index, (pos, statement)) = vm
        .block
        .iter()
        .find_position(|it| it.0 == target)
        .ok_or(Error::invalid_data("Loop didn't resolve any statements"))?;

    if let Statement::If { condition, .. } = statement {
        create_loop(vm, index, *pos, condition.clone());
        Ok(())
    } else {
        Err(Error::invalid_data("Loop didn't resolve conditional jump"))
    }
}

fn is_assignment_statement(statement: &&Statement) -> bool {
    matches!(
        statement,
        Statement::ExpressionStatement(_) | Statement::SetVariable { .. }
    )
}

fn create_loop(vm: &mut VirtualMachine, index: usize, pos: usize, condition: Expression) {
    let mut loop_block: Vec<Statement> = vm.block.drain(index..).map(|it| it.1).collect();
    loop_block.remove(0);

    let satisfies_for_loop = loop_block
        .last()
        .filter(is_assignment_statement)
        .and(vm.block.last())
        .map(|it| &it.1)
        .filter(is_assignment_statement)
        .is_some();

    if satisfies_for_loop {
        let (pos, declare) = vm.block.pop().unwrap();
        let increment = loop_block.pop().unwrap();
        vm.block.push((
            pos,
            Statement::For {
                increment: Box::new(increment),
                declare: Box::new(declare),
                condition,
                block: Block { body: loop_block },
            },
        ));
    } else {
        vm.block.push((
            pos,
            Statement::While {
                condition,
                block: Block { body: loop_block },
            },
        ));
    }
}
