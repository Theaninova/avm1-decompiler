use crate::ast::binary_expr::{BinaryExpression, BinaryExpressionType};
use crate::ast::expr::{
    ASExpression, ASFunctionCallExpression, ASGetMemberExpression, ASReferenceExpression,
};
use crate::ast::statement::{
    DefineLocal, FunctionDeclaration, SetMember, SetVariable, Statement, StoreRegister,
};
use crate::ast::unary_expr::{UnaryExpression, UnaryExpressionType};
use crate::ast::variant::Variant;
use crate::ast::ASIdentifier;
use byteorder::{LittleEndian, ReadBytesExt};
use swf::avm1::read::Reader;
use swf::avm1::types::{Action, DefineFunction2, Value};
use swf::error::Error;
use swf::SwfStr;

pub struct Avm1Decompiler<'a> {
    pub reader: Reader<'a>,
    pub symbols: &'a [&'a str],
    pub scope_stack: Vec<u32>,
    pub stack: Vec<ASExpression>,
    pub register: Vec<ASExpression>,
}

impl<'a> Avm1Decompiler<'a> {
    fn with_register(data: &'a [u8], symbols: &'a [&'a str], register: Vec<ASExpression>) -> Self {
        return Self {
            symbols,
            reader: Reader::new(data, 1),
            register,
            scope_stack: Vec::new(),
            stack: Vec::new(),
        };
    }

    pub fn new(data: &'a [u8], symbols: &'a [&'a str]) -> Self {
        return Self {
            symbols,
            reader: Reader::new(data, 1),
            stack: Vec::new(),
            scope_stack: Vec::new(),
            register: Vec::new(),
        };
    }

    pub fn decompile(&mut self) -> Result<Vec<Statement>, Error> {
        let mut statements = Vec::<Statement>::new();
        let mut action = self.reader.read_action();
        while let Ok(a) = action {
            match a {
                Action::DefineFunction2(define) => statements.push(Statement::FunctionDeclaration(
                    self.decompile_define_function(define)?,
                )),
                Action::DefineFunction(define) => statements.push(Statement::FunctionDeclaration(
                    self.decompile_define_function(define.into())?,
                )),
                Action::CallFunction => {
                    let name = ASReferenceExpression::from_expression(self.stack.pop().unwrap());
                    let num_args = match self.stack.pop().unwrap() {
                        ASExpression::Literal(Variant::Int(i)) => i as usize,
                        _ => {
                            eprintln!("Tried calling a function with non-constant arg count");
                            0
                        }
                    };
                    let mut args = Vec::<ASExpression>::with_capacity(num_args);
                    for _ in 0..num_args {
                        args.push(self.stack.pop().unwrap());
                    }

                    self.stack
                        .push(ASExpression::CallFunction(ASFunctionCallExpression {
                            name,
                            args,
                        }))
                }
                Action::Push(push) => {
                    for value in push.values.iter() {
                        self.stack.push(match value {
                            Value::Undefined => ASExpression::Literal(Variant::Undefined),
                            Value::Null => ASExpression::Literal(Variant::Null),
                            Value::Bool(val) => ASExpression::Literal(Variant::Bool(*val)),
                            Value::Int(val) => ASExpression::Literal(Variant::Int(*val)),
                            Value::Float(val) => ASExpression::Literal(Variant::Float(*val)),
                            Value::Double(val) => ASExpression::Literal(Variant::Double(*val)),
                            Value::ConstantPool(id) => ASExpression::Literal(Variant::String(
                                self.symbols[*id as usize].to_string(),
                            )),
                            Value::Register(id) => {
                                ASExpression::Reference(ASReferenceExpression::Register(*id))
                            }
                            Value::Str(val) => {
                                ASExpression::Literal(Variant::String(self.fill_string(val)))
                            }
                        })
                    }
                }
                Action::PushDuplicate => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(value.clone());
                    self.stack.push(value);
                }
                Action::StoreRegister(store) => {
                    let value = self.stack.last().unwrap();
                    self.register[store.register as usize] = value.clone();
                    statements.push(Statement::StoreRegister(StoreRegister {
                        id: store.register,
                        value: value.clone(),
                    }))
                }
                Action::Pop => {
                    let expr = self.stack.pop().unwrap();
                    if let ASExpression::CallFunction(_) = expr {
                        statements.push(Statement::ExpressionStatement(expr))
                    }
                }
                Action::GetMember => {
                    let name = ASReferenceExpression::from_expression(self.stack.pop().unwrap());
                    let object = ASReferenceExpression::from_expression(self.stack.pop().unwrap());
                    self.stack
                        .push(ASExpression::GetMember(ASGetMemberExpression {
                            name: Box::new(name),
                            object: Box::new(object),
                        }))
                }
                Action::InitArray => {
                    let num_elements = self.stack.pop().unwrap();

                    let value = ASExpression::Literal(Variant::Array(match num_elements {
                        ASExpression::Literal(Variant::Int(i)) => {
                            (0..i).map(|_| self.stack.pop().unwrap()).collect()
                        }
                        _ => {
                            eprintln!("Tried to init array with non-constant size");
                            vec![]
                        }
                    }));
                    self.stack.push(value);
                }
                Action::InitObject => {
                    let num_props = self.stack.pop().unwrap();
                    match num_props {
                        ASExpression::Literal(Variant::Int(i)) => {
                            let mut map =
                                Vec::<(ASExpression, ASExpression)>::with_capacity(i as usize);
                            for _ in 0..i {
                                let value = self.stack.pop().unwrap();
                                let name = self.stack.pop().unwrap();
                                map.push((name, value));
                            }
                            self.stack.push(ASExpression::Literal(Variant::Object(map)))
                        }
                        _ => {
                            eprintln!("Tried to init an object with non-constant size")
                        }
                    }
                }
                Action::GetVariable => {
                    let path = ASReferenceExpression::from_expression(self.stack.pop().unwrap());
                    self.stack.push(ASExpression::Reference(path))
                }
                Action::SetVariable => {
                    let value = self.stack.pop().unwrap();
                    let path = ASReferenceExpression::from_expression(self.stack.pop().unwrap());
                    statements.push(Statement::SetVariable(SetVariable {
                        left: path,
                        right: value,
                    }))
                }
                Action::DefineLocal => {
                    let value = self.stack.pop().unwrap();
                    let name = ASReferenceExpression::from_expression(self.stack.pop().unwrap());

                    statements.push(Statement::DefineLocal(DefineLocal {
                        left: name,
                        right: value,
                    }))
                }
                Action::SetMember => {
                    let value = self.stack.pop().unwrap();
                    let name = ASReferenceExpression::from_expression(self.stack.pop().unwrap());
                    let object = ASReferenceExpression::from_expression(self.stack.pop().unwrap());

                    statements.push(Statement::SetMember(SetMember {
                        object,
                        name,
                        value,
                    }))
                }
                Action::End => statements.push(Statement::Return(None)),
                Action::Return => {
                    let value = self.stack.pop().unwrap();
                    statements.push(Statement::Return(Some(value)))
                }

                Action::Subtract => self.decompile_binary_expr(BinaryExpressionType::Subtract),
                Action::Add | Action::Add2 | Action::StringAdd => {
                    self.decompile_binary_expr(BinaryExpressionType::Add)
                }
                Action::Divide => self.decompile_binary_expr(BinaryExpressionType::Divide),
                Action::Multiply => self.decompile_binary_expr(BinaryExpressionType::Multiply),
                Action::Modulo => self.decompile_binary_expr(BinaryExpressionType::Modulo),

                Action::BitXor => self.decompile_binary_expr(BinaryExpressionType::BitXor),
                Action::BitAnd => self.decompile_binary_expr(BinaryExpressionType::BitAnd),
                Action::BitOr => self.decompile_binary_expr(BinaryExpressionType::BitOr),
                Action::BitURShift => self.decompile_binary_expr(BinaryExpressionType::BitURShift),
                Action::BitRShift => self.decompile_binary_expr(BinaryExpressionType::BitRShift),
                Action::BitLShift => self.decompile_binary_expr(BinaryExpressionType::BitLShift),

                Action::Greater | Action::StringGreater => {
                    self.decompile_binary_expr(BinaryExpressionType::Greater)
                }
                Action::Less2 | Action::Less | Action::StringLess => {
                    self.decompile_binary_expr(BinaryExpressionType::Less)
                }
                Action::Not => self.decompile_unary_expr(UnaryExpressionType::Not),
                Action::Or => self.decompile_binary_expr(BinaryExpressionType::LogicalOr),
                Action::And => self.decompile_binary_expr(BinaryExpressionType::LogicalAnd),
                Action::StrictEquals => {
                    self.decompile_binary_expr(BinaryExpressionType::StrictEquals)
                }
                Action::Equals | Action::Equals2 | Action::StringEquals => {
                    self.decompile_binary_expr(BinaryExpressionType::Equals)
                }

                Action::Increment => self.decompile_unary_expr(UnaryExpressionType::Increment),
                Action::Decrement => self.decompile_unary_expr(UnaryExpressionType::Decrement),

                _ => {
                    eprintln!("Not implemented: {:?}", a);
                    statements.push(Statement::UnknownStatement(format!("{:?}", a)))
                }
            }
            action = self.reader.read_action();
        }

        let error = action.expect_err("what");
        match error {
            Error::IoError(_error) => Ok(statements),
            _ => Ok(statements),
        }
    }
    fn decompile_unary_expr(&mut self, expression_type: UnaryExpressionType) {
        let target = self.stack.pop().unwrap();
        self.stack.push(ASExpression::Unary(UnaryExpression {
            target: Box::new(target),
            expression_type,
        }))
    }
    fn decompile_binary_expr(&mut self, expression_type: BinaryExpressionType) {
        let right = self.stack.pop().unwrap();
        let left = self.stack.pop().unwrap();
        self.stack.push(ASExpression::Binary(BinaryExpression {
            left: Box::new(left),
            right: Box::new(right),
            expression_type,
        }))
    }

    fn decompile_define_function(
        &mut self,
        function: DefineFunction2,
    ) -> Result<FunctionDeclaration, Error> {
        let mut register = Vec::<ASExpression>::with_capacity(function.register_count as usize);
        register.push(ASExpression::Reference(ASReferenceExpression::Identifier(
            "this".to_string(),
        )));
        for _ in 1..function.register_count {
            register.push(ASExpression::Literal(Variant::Uninitialized));
        }
        let mut params = Vec::<ASReferenceExpression>::with_capacity(function.params.len());

        for param in function.params.iter() {
            let result = ASReferenceExpression::Identifier(self.fill_string(param.name));
            let register_index = param.register_index.unwrap().get();
            params.push(result.clone());
            register[register_index as usize] = ASExpression::Reference(result);
        }

        let body = {
            let mut body_transpiler =
                Avm1Decompiler::with_register(function.actions, self.symbols, register);
            body_transpiler.decompile().unwrap()
        };

        Ok(FunctionDeclaration {
            identifier: ASIdentifier {
                name: self.fill_string(function.name),
            },
            flags: function.flags,
            parameters: params,
            body,
        })
    }

    fn fill_string(&self, string: &SwfStr) -> String {
        let mut bytes = string.as_bytes();
        return match bytes.len() {
            0 => self.symbols[0].to_string(),
            1 => self.symbols[bytes.read_u8().unwrap() as usize].to_string(),
            2 => self.symbols[bytes.read_u16::<LittleEndian>().unwrap() as usize].to_string(),
            _ => panic!("Bad error"),
        };
    }
}
