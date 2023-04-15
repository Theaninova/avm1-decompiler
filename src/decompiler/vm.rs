use crate::ast::block::Block;
use crate::ast::expr::{Expression, SuperpositionExpression};
use crate::ast::statement::Statement;
use crate::ast::variant::Variant;
use crate::ast::variant::Variant::Uninitialized;
use crate::decompiler::read::read;
use crate::decompiler::VmData;
use itertools::Itertools;
use std::borrow::Cow;
use std::ptr::write;
use swf::avm1::read::Reader;
use swf::avm1::types::Action;
use swf::avm2::types::Op;
use swf::error::{Error, Result};
use swf::extensions::ReadSwfExt;

impl<'a> From<VmData<'a>> for VirtualMachine<'a> {
    fn from(value: VmData<'a>) -> Self {
        VirtualMachine {
            reader: Reader::new(value.bytecode, 1),
            stack: vec![],
            block: vec![],
            control_flow_graph: vec![ControlFlowNode::Entry { next: 0 }],
            data: value,
            offset: 0,
        }
    }
}

pub enum ControlFlowNode {
    Branch {
        parent: usize,
        condition: Expression,
        if_true: usize,
        if_false: usize,
    },
    Join {
        branches: Vec<usize>,
    },
    Return {
        parent: usize,
    },
    Entry {
        next: usize,
    },
}

pub struct VirtualMachine<'a> {
    stack: Vec<(usize, Expression)>,
    block: Vec<(usize, Statement)>,
    control_flow_graph: Vec<ControlFlowNode>,
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

        if offset < 0 {
            let statement = self.block.iter().find_position(|it| it.0 == target);

            let condition = if let Some((
                index,
                (
                    pos,
                    Statement::If {
                        condition,
                        true_branch,
                        false_branch,
                    },
                ),
            )) = statement
            {
                Some((index, *pos, condition.clone()))
            } else {
                None
            };

            if let Some((index, pos, condition)) = condition {
                let mut loop_block: Vec<Statement> =
                    self.block.drain((index + 1)..).map(|it| it.1).collect();
                self.block.pop().unwrap();

                match (self.block.pop(), loop_block.pop()) {
                    (
                        Some((pos, Statement::ExpressionStatement(declare))),
                        Some(Statement::ExpressionStatement(increment)),
                    ) => {
                        self.block.push((
                            pos,
                            Statement::For {
                                increment: increment.clone(),
                                declare: declare.clone(),
                                condition,
                                block: Block { body: loop_block },
                            },
                        ));
                    }
                    (declare, increment) => {
                        if let Some(declare) = declare {
                            self.block.push(declare)
                        };
                        if let Some(increment) = increment {
                            loop_block.push(increment)
                        };
                        self.block.push((
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
        println!(
            ">> [{:04}-{:04}] return {}",
            position,
            actual_position,
            value.unwrap_or(Expression::Literal(Variant::Uninitialized))
        );
    }

    pub fn get_constant(&mut self, id: usize) -> String {
        self.data.constant_pool[id].clone()
    }

    pub fn finalize(mut self) -> Vec<Statement> {
        if !self.stack.is_empty() {
            eprintln!("{} remaining items on the stack", self.stack.len())
        }
        println!("-----");
        // TODO
        // for jump in self.jump_table {
        //     println!(">> {:?}", jump);
        // }
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
