use crate::ast::block::Block;
use crate::ast::statement::Statement;
use crate::decompiler::vm::VirtualMachine;
use itertools::Itertools;

pub fn resolve_loop(vm: &mut VirtualMachine, target: usize) {
    let statement = vm.block.iter().find_position(|it| it.0 == target);
    let condition = if let Some((
        index,
        (
            pos,
            Statement::If {
                condition,
                true_branch: _,
                false_branch: _,
            },
        ),
    )) = statement
    {
        Some((index, *pos, condition.clone()))
    } else {
        None
    };

    if let Some((index, pos, condition)) = condition {
        let mut loop_block: Vec<Statement> = vm.block.drain(index..).map(|it| it.1).collect();
        loop_block.remove(0);

        match (vm.block.pop(), loop_block.pop()) {
            (Some((pos, declare)), Some(increment))
                if matches!(
                    declare,
                    Statement::ExpressionStatement(_) | Statement::SetVariable { .. }
                ) && matches!(
                    increment,
                    Statement::ExpressionStatement(_) | Statement::SetVariable { .. }
                ) =>
            {
                vm.block.push((
                    pos,
                    Statement::For {
                        increment: Box::new(increment),
                        declare: Box::new(declare),
                        condition,
                        block: Block { body: loop_block },
                    },
                ));
            }
            (declare, increment) => {
                if let Some(declare) = declare {
                    vm.block.push(declare)
                };
                if let Some(increment) = increment {
                    loop_block.push(increment)
                };
                vm.block.push((
                    pos,
                    Statement::While {
                        condition,
                        block: Block { body: loop_block },
                    },
                ));
            }
        }
    } else {
        println!("❌ Loop didn't resolve")
    }
}
