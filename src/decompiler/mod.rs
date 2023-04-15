mod components;
mod read;
mod vm;
use crate::ast::binary_expr::BinaryExpressionType;
use crate::ast::block::Block;
use crate::ast::expr::{Expression, ReferenceExpression, UnaryExpressionType};
use crate::ast::statement::Statement;
use crate::ast::variant::Variant;
use crate::decompiler::components::binary_expression::decompile_binary_expr;
use crate::decompiler::components::functions::decompile_define_function;
use crate::decompiler::components::unary_expression::decompile_unary_expr;
use crate::decompiler::vm::VirtualMachine;
use itertools::Itertools;
use std::borrow::Cow;
use swf::avm1::types::{Action, Value};
use swf::error::{Error, Result};
use swf::UTF_8;

#[derive(Debug, Default)]
pub struct VmData<'a> {
    pub bytecode: &'a [u8],
    pub constant_pool: &'a [String],
    pub strict: bool,
    pub registers: Vec<Expression>,
}

pub fn decompile(data: VmData) -> Result<Vec<Statement>> {
    internal_decompile(data.into())
}

fn internal_decompile(mut vm: VirtualMachine) -> Result<Vec<Statement>> {
    loop {
        match vm.read_action()? {
            Action::DefineFunction2(define) => {
                let function = decompile_define_function(&mut vm, define)?;
                if let Expression::Function {
                    identifier: None, ..
                } = function
                {
                    vm.push(function)
                } else {
                    vm.append_statement(Statement::ExpressionStatement(function))
                }
            }
            Action::DefineFunction(define) => {
                let function = decompile_define_function(&mut vm, define.into())?;
                if let Expression::Function {
                    identifier: None, ..
                } = function
                {
                    vm.push(function)
                } else {
                    vm.append_statement(Statement::ExpressionStatement(function))
                }
            }
            Action::CallFunction => {
                let name = ReferenceExpression::from_expression(vm.pop()?);
                let num_args = match vm.pop()? {
                    Expression::Literal(Variant::Int(i)) => i as usize,
                    _ => {
                        eprintln!("Tried calling a function with non-constant arg count");
                        0
                    }
                };
                let args = vm.pop_len(num_args)?;
                vm.push(Expression::CallFunction { name, args })
            }
            Action::CallMethod => {
                let name = ReferenceExpression::from_expression(vm.pop()?);
                let object = ReferenceExpression::from_expression(vm.pop()?);
                let num_args = match vm.pop()? {
                    Expression::Literal(Variant::Int(i)) => i as usize,
                    _ => {
                        eprintln!("Tried calling a function with non-constant arg count");
                        0
                    }
                };
                let args = vm.pop_len(num_args)?;
                vm.push(Expression::CallMethod { name, object, args })
            }
            Action::Push(push) => {
                for value in push.values.iter() {
                    let expression = match value {
                        Value::Undefined => Expression::Literal(Variant::Undefined),
                        Value::Null => Expression::Literal(Variant::Null),
                        Value::Bool(val) => Expression::Literal(Variant::Bool(*val)),
                        Value::Int(val) => Expression::Literal(Variant::Int(*val)),
                        Value::Float(val) => Expression::Literal(Variant::Float(*val)),
                        Value::Double(val) => Expression::Literal(Variant::Double(*val)),
                        Value::ConstantPool(id) => {
                            Expression::Literal(Variant::String(vm.get_constant(*id as usize)))
                        }
                        Value::Register(id) => {
                            Expression::Reference(ReferenceExpression::Register(*id))
                        }
                        Value::Str(val) => {
                            Expression::Literal(Variant::String(val.to_string_lossy(UTF_8)))
                        }
                    };
                    vm.push(expression)
                }
            }
            Action::PushDuplicate => {
                let value = vm.pop()?;
                vm.push(value.clone());
                vm.push(value);
            }
            Action::StoreRegister(store) => {
                let value = vm.pop()?;
                vm.store(store.register, value.clone())?;
                vm.push(Expression::StoreRegister {
                    id: store.register,
                    value: Box::new(value.clone()),
                })
            }
            Action::If(target) => {
                // negate the expression
                decompile_unary_expr(&mut vm, UnaryExpressionType::Not)?;
                let condition = vm.pop()?;
                vm.jump(target.offset, Some(condition.clone()));
                // let true_branch = internal_decompile(vm.resolve_jump(target.offset))?;
                // let false_branch = internal_decompile(vm.resolve_jump(target.offset))?;
                vm.append_statement(Statement::If {
                    condition,
                    true_branch: Block {
                        body: Vec::new(), /*true_branch*/
                    },
                    false_branch: Block {
                        body: Vec::new(), /*false_branch*/
                    },
                })
            }
            Action::Jump(target) => {
                vm.jump(target.offset, None);
            }
            Action::Pop => {
                let expr = vm.pop()?;
                vm.append_statement(Statement::ExpressionStatement(expr))
            }
            Action::ToInteger => {
                let value = vm.pop()?;
                vm.push(Expression::Unary {
                    target: Box::new(value),
                    expression_type: UnaryExpressionType::ToInteger,
                })
            }
            Action::ToString => {
                let value = vm.pop()?;
                vm.push(Expression::Unary {
                    target: Box::new(value),
                    expression_type: UnaryExpressionType::ToString,
                })
            }
            Action::ToNumber => {
                let value = vm.pop()?;
                vm.push(Expression::Unary {
                    target: Box::new(value),
                    expression_type: UnaryExpressionType::ToNumber,
                })
            }
            Action::GetMember => {
                let name = ReferenceExpression::from_expression(vm.pop()?);
                let object = ReferenceExpression::from_expression(vm.pop()?);
                vm.push(Expression::GetMember { name, object })
            }
            Action::GetProperty => {
                let name = ReferenceExpression::from_expression(vm.pop()?);
                let object = ReferenceExpression::from_expression(vm.pop()?);
                vm.push(Expression::GetMember { name, object })
            }
            Action::InitArray => {
                let elements = if let Expression::Literal(Variant::Int(i)) = vm.pop()? {
                    vm.pop_len(i as usize)?
                } else {
                    return Err(Error::InvalidData(Cow::from(
                        "Init array must have constant-size elements",
                    )));
                };
                vm.push(Expression::Literal(Variant::Array(elements)));
            }
            Action::InitObject => {
                let props: Vec<(Expression, Expression)> =
                    if let Expression::Literal(Variant::Int(i)) = vm.pop()? {
                        vm.pop_len(i as usize * 2)?
                            .into_iter()
                            .rev()
                            .tuples()
                            .collect()
                    } else {
                        return Err(Error::InvalidData(Cow::from(
                            "Init object must have constant-size elements",
                        )));
                    };
                vm.push(Expression::Literal(Variant::Object(props)));
            }
            Action::GetVariable => {
                let path = match vm.pop()? {
                    Expression::Literal(Variant::String(str)) => ReferenceExpression::Variable(str),
                    it => ReferenceExpression::from_expression(it),
                };
                vm.push(Expression::Reference(path))
            }
            Action::SetVariable => {
                let value = vm.pop()?;
                let path = ReferenceExpression::from_expression(vm.pop()?);
                vm.append_statement(Statement::SetVariable {
                    left: path,
                    right: value.into(),
                })
            }
            Action::Trace => {
                let expr = vm.pop()?;
                vm.append_statement(Statement::Trace(expr))
            }
            Action::DefineLocal => {
                let right = vm.pop()?;
                let left = ReferenceExpression::from_expression(vm.pop()?);

                vm.append_statement(Statement::DefineLocal { left, right })
            }
            Action::DefineLocal2 => {
                let name = ReferenceExpression::from_expression(vm.pop()?);
                vm.append_statement(Statement::DeclareLocal { name })
            }
            Action::SetMember => {
                let value = vm.pop()?;
                let name = ReferenceExpression::from_expression(vm.pop()?);
                let object = ReferenceExpression::from_expression(vm.pop()?);

                vm.append_statement(Statement::SetMember {
                    object,
                    name,
                    value,
                })
            }
            Action::Return => {
                let value = vm.pop()?;
                vm.jump_return(Some(value));
            }

            Action::Subtract => decompile_binary_expr(&mut vm, BinaryExpressionType::Subtract)?,
            Action::Add | Action::Add2 | Action::StringAdd => {
                decompile_binary_expr(&mut vm, BinaryExpressionType::Add)?
            }
            Action::Divide => decompile_binary_expr(&mut vm, BinaryExpressionType::Divide)?,
            Action::Multiply => decompile_binary_expr(&mut vm, BinaryExpressionType::Multiply)?,
            Action::Modulo => decompile_binary_expr(&mut vm, BinaryExpressionType::Modulo)?,

            Action::BitXor => decompile_binary_expr(&mut vm, BinaryExpressionType::BitXor)?,
            Action::BitAnd => decompile_binary_expr(&mut vm, BinaryExpressionType::BitAnd)?,
            Action::BitOr => decompile_binary_expr(&mut vm, BinaryExpressionType::BitOr)?,
            Action::BitURShift => decompile_binary_expr(&mut vm, BinaryExpressionType::BitURShift)?,
            Action::BitRShift => decompile_binary_expr(&mut vm, BinaryExpressionType::BitRShift)?,
            Action::BitLShift => decompile_binary_expr(&mut vm, BinaryExpressionType::BitLShift)?,

            Action::Greater | Action::StringGreater => {
                decompile_binary_expr(&mut vm, BinaryExpressionType::Greater)?
            }
            Action::Less2 | Action::Less | Action::StringLess => {
                decompile_binary_expr(&mut vm, BinaryExpressionType::Less)?
            }
            Action::Not => decompile_unary_expr(&mut vm, UnaryExpressionType::Not)?,
            Action::Or => decompile_binary_expr(&mut vm, BinaryExpressionType::LogicalOr)?,
            Action::And => decompile_binary_expr(&mut vm, BinaryExpressionType::LogicalAnd)?,
            Action::StrictEquals => {
                decompile_binary_expr(&mut vm, BinaryExpressionType::StrictEquals)?
            }
            Action::Equals | Action::Equals2 | Action::StringEquals => {
                decompile_binary_expr(&mut vm, BinaryExpressionType::Equals)?
            }

            Action::Increment => decompile_unary_expr(&mut vm, UnaryExpressionType::Increment)?,
            Action::Decrement => decompile_unary_expr(&mut vm, UnaryExpressionType::Decrement)?,

            Action::End => return Ok(vm.finalize()),
            Action::Stop => vm.append_statement(Statement::Stop),
            Action::GotoLabel(label) => {
                vm.append_statement(Statement::GotoLabel(label.label.to_string_lossy(UTF_8)))
            }
            Action::GotoFrame(frame) => vm.append_statement(Statement::GotoFrame(frame.frame)),
            Action::Play => vm.append_statement(Statement::Play),

            action => {
                eprintln!("Not implemented: {:?}", action);
                vm.append_statement(Statement::UnknownStatement(format!("{:?}", action)))
            }
        }
    }
}
