use crate::ast::expr::Expression;
use crate::ast::statement::Statement;
use crate::ast::variant::Variant;
use crate::ast::variant::Variant::Uninitialized;
use crate::decompiler::read::read;
use crate::decompiler::VmData;
use std::borrow::Cow;
use swf::avm1::read::Reader;
use swf::avm1::types::Action;
use swf::avm2::types::Op;
use swf::error::{Error, Result};
use swf::extensions::ReadSwfExt;

impl<'a> From<VmData<'a>> for VirtualMachine<'a> {
    fn from(value: VmData<'a>) -> Self {
        VirtualMachine {
            reader: Reader::new(value.bytecode, 1),
            stack: Vec::new(),
            block: Vec::new(),
            states: vec![VmState {
                offset: 0,
                next_change: None,
                active: true,
                cursor: 0,
                condition: None,
            }],
            jump_table: Vec::new(),
            data: value,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone)]
enum VmJump {
    Conditional {
        position: usize,
        actual_position: usize,
        target: usize,
        expr: Expression,
    },
    Jump {
        position: usize,
        target: usize,
    },
    Return {
        position: usize,
        actual_position: usize,
        expr: Option<Expression>,
    },
}

pub struct Superposition {
    id: usize,
    next_change: Option<usize>,
    active: bool,
    cursor: usize,
    condition_id: Option<usize>,
}

pub struct LinearSuperpositionStack {
    superpositions: Vec<Superposition>,
    stack: Vec<Vec<(usize, Expression)>>,
}

impl LinearSuperpositionStack {
    /*pub fn pop(&mut self, position: usize) -> Option<Expression> {
        let active = self.tick(position);
        let mut values = Vec::new();
        for superposition in active {
            values.push(if let Some(values) = self.stack.get(superposition.cursor) {
                values.iter().find(|id| (**id).0 == superposition.id)
            } else {
                return None;
            });
        }
        if values.len() == 1 {
            return values[0];
        }
        values
    }

    fn tick(&mut self, position: usize) -> Vec<&Superposition> {
        let mut active = Vec::new();
        for mut superposition in self.superpositions {
            if let Some(next_change) = superposition.next_change {
                if next_change == position {
                    superposition.active = !superposition.active;
                }
            }
            if superposition.active {
                active.push(&superposition);
            }
        }
        active
    }*/
}

pub struct VmState {
    offset: usize,
    next_change: Option<usize>,
    active: bool,
    cursor: usize, // position in the stack from the bottom
    condition: Option<(usize, Expression)>,
}

pub struct VirtualMachine<'a> {
    states: Vec<VmState>,
    jump_table: Vec<VmJump>,
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
        } else if self.data.strict {
            Err(Error::InvalidData(Cow::from(
                "Tried to set non-existent register",
            )))
        } else {
            while self.data.registers.len() < i {
                self.data.registers.push(Expression::Literal(Uninitialized));
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
        self.jump_table.push(if let Some(expr) = condition {
            VmJump::Conditional {
                expr,
                position,
                actual_position,
                target,
            }
        } else {
            VmJump::Jump { position, target }
        })
    }

    pub fn jump_return(&mut self, value: Option<Expression>) {
        self.jump_table.push(VmJump::Return {
            expr: value,
            position: self.offset,
            actual_position: self.reader.pos(self.data.bytecode),
        })
    }

    pub fn get_constant(&mut self, id: usize) -> String {
        self.data.constant_pool[id].clone()
    }

    pub fn finalize(mut self) -> Vec<Statement> {
        if !self.stack.is_empty() {
            eprintln!("{} remaining items on the stack", self.stack.len())
        }
        println!("-----");
        for jump in self.jump_table {
            println!(">> {:?}", jump);
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
