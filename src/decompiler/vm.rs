use crate::ast::expr::Expression;
use crate::ast::statement::Statement;
use crate::ast::variant::Variant;
use crate::decompiler::read::read;
use crate::decompiler::vm::loops::resolve_loop;
use crate::decompiler::VmData;
use std::borrow::Cow;
use swf::avm1::read::Reader;
use swf::avm1::types::Action;
use swf::error::{Error, Result};
use swf::extensions::ReadSwfExt;

mod loops;

impl<'a> From<VmData<'a>> for VirtualMachine<'a> {
    fn from(value: VmData<'a>) -> Self {
        VirtualMachine {
            reader: Reader::new(value.bytecode, 1),
            stack: vec![],
            block: vec![],
            pending_branches: vec![],
            data: value,
            offset: 0,
        }
    }
}

pub struct VirtualMachine<'a> {
    stack: Vec<(usize, Expression)>,
    block: Vec<(usize, Statement)>,
    pending_branches: Vec<(usize, usize)>,
    reader: Reader<'a>,
    offset: usize,
    pub data: VmData<'a>,
}

impl<'a> VirtualMachine<'a> {
    pub fn pop(&mut self) -> Result<Expression> {
        let (new_offset, value) = if self.data.strict {
            self.stack
                .pop()
                .ok_or(Error::InvalidData(Cow::from("Tried to pop empty stack")))?
        } else if let Some(value) = self.stack.pop() {
            value
        } else {
            eprintln!("Popped empty stack");
            (0, Expression::Literal(Variant::Uninitialized))
        };
        self.offset = new_offset;
        Ok(value)
    }

    pub fn pop_len(&mut self, len: usize) -> Result<Vec<Expression>> {
        let mut vec = Vec::<Expression>::with_capacity(len);
        for _ in 0..len {
            vec.push(self.pop()?);
        }
        Ok(vec)
    }

    pub fn store(&mut self, register_id: u8, value: Expression) -> Result<()> {
        let i = register_id as usize;
        if (0..self.data.registers.len()).contains(&i) {
            self.data.registers[i] = value;
            Ok(())
        } else if self.data.strict {
            Err(Error::InvalidData(Cow::from(
                "Tried to set non-existent register",
            )))
        } else {
            while self.data.registers.len() < i {
                self.data
                    .registers
                    .push(Expression::Literal(Variant::Uninitialized));
            }
            self.data.registers.push(value);
            Ok(())
        }
    }

    pub fn push(&mut self, expression: Expression) {
        self.stack.push((self.offset, expression));
    }

    pub fn append_statement(&mut self, statement: Statement) {
        self.block.push((self.offset, statement));
    }

    pub fn read_action(&mut self) -> Result<Action<'a>> {
        self.offset = self.reader.pos(self.data.bytecode);
        read(
            &mut self.reader,
            self.data.bytecode,
            self.data.constant_pool,
        )
    }

    pub fn jump(&mut self, offset: i16, condition: Option<Expression>) {
        let actual_position = self.reader.pos(self.data.bytecode);
        let position = self.offset;
        let target = (actual_position as i64 + offset as i64) as usize;

        if offset < 0 {
            resolve_loop(self, target);
        }

        let jump = if offset < 0 {
            format!(
                ">> {:04} <- [{:04}-{:04}]",
                target, position, actual_position
            )
        } else {
            format!(
                ">> [{:04}-{:04}] -> {:04}",
                position, actual_position, target
            )
        };

        if let Some(condition) = condition {
            println!("{} if not {}", jump, condition);
        } else {
            println!("{}", jump)
        }
    }

    pub fn jump_return(&mut self, value: Option<Expression>) {
        let actual_position = self.reader.pos(self.data.bytecode);
        let position = self.offset;
        if let Some(value) = value {
            println!(
                ">> [{:04}-{:04}] return {}",
                position, actual_position, value,
            );
            self.append_statement(Statement::Return(Some(value)));
        } else {
            println!(">> [{:04}-{:04}] return", position, actual_position,);
            self.append_statement(Statement::Return(None))
        }
    }

    pub fn get_constant(&mut self, id: usize) -> String {
        self.data.constant_pool[id].clone()
    }

    pub fn finalize(mut self) -> Vec<Statement> {
        if !self.stack.is_empty() {
            eprintln!("{} remaining items on the stack", self.stack.len())
        }
        println!("-----");
        let mut dangling_stack = self
            .stack
            .into_iter()
            .rev()
            .map(|(pos, expr)| (pos, Statement::DanglingStack(expr)))
            .collect();
        self.block.append(&mut dangling_stack);
        self.block
            .into_iter()
            .map(|(_, statement)| statement)
            .collect()
    }
}
