use crate::ast::expr::Expression;
use crate::ast::statement::Statement;
use crate::ast::variant::Variant;
use crate::decompiler::read::read;
use crate::decompiler::VmData;
use std::borrow::Cow;
use swf::avm1::read::Reader;
use swf::avm1::types::Action;
use swf::error::{Error, Result};
use swf::extensions::ReadSwfExt;

impl<'a> From<VmData<'a>> for VirtualMachine<'a> {
    fn from(value: VmData<'a>) -> Self {
        VirtualMachine {
            reader: Reader::new(value.bytecode, 1),
            stack: Vec::new(),
            block: Vec::new(),
            data: value,
            offset: 0,
        }
    }
}

pub struct VirtualMachine<'a> {
    stack: Vec<(usize, Expression)>,
    block: Vec<(usize, Statement)>,
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
        } else {
            Err(Error::InvalidData(Cow::from(
                "Tried to set non-existent register",
            )))
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

    pub fn get_constant(&mut self, id: usize) -> String {
        self.data.constant_pool[id].clone()
    }

    pub fn finalize(self) -> Vec<Statement> {
        if !self.stack.is_empty() {
            eprintln!("{} remaining items on the stack", self.stack.len())
        }
        self.block
            .into_iter()
            .map(|(_, statement)| statement)
            .collect()
    }
}
