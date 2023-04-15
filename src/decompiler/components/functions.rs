use crate::ast::block::Block;
use crate::ast::expr::{Expression, ReferenceExpression};
use crate::ast::variant::Variant;
use crate::decompiler::vm::VirtualMachine;
use crate::decompiler::{decompile, VmData};
use std::num::NonZeroU8;
use swf::avm1::types::DefineFunction2;
use swf::error::Result;
use swf::UTF_8;

pub fn decompile_define_function(
    vm: &mut VirtualMachine,
    function: DefineFunction2,
) -> Result<Expression> {
    let mut registers: Vec<Expression> = (0..function.register_count)
        .map(|_| Expression::Literal(Variant::Uninitialized))
        .collect();
    let mut params = Vec::<ReferenceExpression>::with_capacity(function.params.len());

    for param in function.params.into_iter() {
        let name = param.name.to_string_lossy(UTF_8);
        let result = ReferenceExpression::Identifier(name);
        let register_index = if let Some(index) = param.register_index {
            index
        } else {
            NonZeroU8::new(1).unwrap()
        };

        params.push(result.clone());
        registers[register_index.get() as usize] = Expression::Reference(result);
    }

    let body = decompile(VmData {
        bytecode: function.actions,
        registers,
        constant_pool: vm.data.constant_pool,
        strict: vm.data.strict,
    })?;
    let name = function.name.to_string_lossy(UTF_8);
    Ok(Expression::Function {
        identifier: if name.is_empty() { None } else { Some(name) },
        flags: function.flags,
        parameters: params,
        body: Block { body },
    })
}
