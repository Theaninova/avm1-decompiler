mod read;
mod vm;
use crate::ast::binary_expr::{BinaryExpression, BinaryExpressionType};
use crate::ast::expr::{
    ASFunctionCallExpression, ASGetMemberExpression, Expression, ReferenceExpression,
};
use crate::ast::statement::{
    DefineLocal, FunctionDeclaration, SetMember, SetVariable, Statement, StoreRegister,
};
use crate::ast::unary_expr::{UnaryExpression, UnaryExpressionType};
use crate::ast::variant::Variant;
use crate::ast::ASIdentifier;
use crate::decompiler::vm::VirtualMachine;
use itertools::Itertools;
use std::borrow::Cow;
use swf::avm1::types::{Action, DefineFunction2, Value};
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
    let mut vm: VirtualMachine = data.into();

    loop {
        match vm.read_action()? {
            Action::DefineFunction2(define) => {
                let function = decompile_define_function(&mut vm, define)?;
                if function.identifier.is_none() {
                    vm.push(Expression::Function(function))
                } else {
                    vm.append_statement(Statement::FunctionDeclaration(function))
                }
            }
            Action::DefineFunction(define) => {
                let function = decompile_define_function(&mut vm, define.into())?;
                if function.identifier.is_none() {
                    vm.push(Expression::Function(function))
                } else {
                    vm.append_statement(Statement::FunctionDeclaration(function))
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
                vm.push(Expression::CallFunction(ASFunctionCallExpression {
                    name,
                    args,
                }))
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
                vm.append_statement(Statement::StoreRegister(StoreRegister {
                    id: store.register,
                    value: value.clone(),
                }))
            }
            Action::Pop => {
                let expr = vm.pop()?;
                if let Expression::CallFunction(_) = expr {
                    vm.append_statement(Statement::ExpressionStatement(expr))
                }
            }
            Action::GetMember => {
                let name = ReferenceExpression::from_expression(vm.pop()?);
                let object = ReferenceExpression::from_expression(vm.pop()?);
                vm.push(Expression::GetMember(ASGetMemberExpression {
                    name: Box::new(name),
                    object: Box::new(object),
                }))
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
                        vm.pop_len(i as usize * 2)?.into_iter().tuples().collect()
                    } else {
                        return Err(Error::InvalidData(Cow::from(
                            "Init object must have constant-size elements",
                        )));
                    };
                vm.push(Expression::Literal(Variant::Object(props)));
            }
            Action::GetVariable => {
                let path = ReferenceExpression::from_expression(vm.pop()?);
                vm.push(Expression::Reference(path))
            }
            Action::SetVariable => {
                let value = vm.pop()?;
                let path = ReferenceExpression::from_expression(vm.pop()?);
                vm.append_statement(Statement::SetVariable(SetVariable {
                    left: path,
                    right: value,
                }))
            }
            Action::DefineLocal => {
                let value = vm.pop()?;
                let name = ReferenceExpression::from_expression(vm.pop()?);

                vm.append_statement(Statement::DefineLocal(DefineLocal {
                    left: name,
                    right: value,
                }))
            }
            Action::SetMember => {
                let value = vm.pop()?;
                let name = ReferenceExpression::from_expression(vm.pop()?);
                let object = ReferenceExpression::from_expression(vm.pop()?);

                vm.append_statement(Statement::SetMember(SetMember {
                    object,
                    name,
                    value,
                }))
            }
            Action::Return => {
                let value = vm.pop()?;
                vm.append_statement(Statement::Return(Some(value)))
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
            Action::Stop => return Ok(vm.finalize()),

            action => {
                eprintln!("Not implemented: {:?}", action);
                vm.append_statement(Statement::UnknownStatement(format!("{:?}", action)))
            }
        }
    }
}

fn decompile_unary_expr(
    vm: &mut VirtualMachine,
    expression_type: UnaryExpressionType,
) -> Result<()> {
    let target = vm.pop()?;
    vm.push(Expression::Unary(UnaryExpression {
        target: Box::new(target),
        expression_type,
    }));
    Ok(())
}

fn decompile_binary_expr(
    vm: &mut VirtualMachine,
    expression_type: BinaryExpressionType,
) -> Result<()> {
    let right = vm.pop()?;
    let left = vm.pop()?;
    vm.push(Expression::Binary(BinaryExpression {
        left: Box::new(left),
        right: Box::new(right),
        expression_type,
    }));
    Ok(())
}

fn decompile_define_function(
    vm: &mut VirtualMachine,
    function: DefineFunction2,
) -> Result<FunctionDeclaration> {
    let mut registers: Vec<Expression> = (0..function.register_count)
        .map(|_| Expression::Literal(Variant::Uninitialized))
        .collect();
    let mut params = Vec::<ReferenceExpression>::with_capacity(function.params.len());

    for param in function.params.into_iter() {
        let name = param.name.to_string_lossy(UTF_8);
        let result = ReferenceExpression::Identifier(name);
        let register_index = param
            .register_index
            .ok_or(Error::InvalidData(Cow::from("Invalid Register Index")))?;
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
    Ok(FunctionDeclaration {
        identifier: if name.is_empty() {
            None
        } else {
            Some(ASIdentifier { name })
        },
        flags: function.flags,
        parameters: params,
        body,
    })
}
