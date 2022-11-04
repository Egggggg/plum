use crate::{
    ast::InfixOp,
    errors::{Error, TypeErrorCtx},
    interpreter::{SpannedValue, Value, ValueType},
};

// da big SpannedValue operation set
impl SpannedValue {
    pub fn pow(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Num(lhs) => match other.0 {
                Value::Num(rhs) => {
                    if lhs == lhs.trunc() && rhs == rhs.trunc() {
                        Ok(Value::Num(lhs.powi(rhs as i32)))
                    } else {
                        Ok(Value::Num(lhs.powf(rhs)))
                    }
                }
                _ => Err(Error::TypeError {
                    expected: ValueType::Num.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Num,
                        op: InfixOp::Pow,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Num.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Pow },
            }),
        }
    }

    pub fn mul(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            // idk why i have to clone here but it wont compile if i dont
            Value::Num(lhs) => match other.0.clone() {
                Value::Num(rhs) => Ok(Value::Num(lhs * rhs)),
                Value::String(rhs) => {
                    if lhs == lhs.trunc() {
                        Ok(Value::String(rhs.repeat(lhs as usize)))
                    } else {
                        Err(Error::TypeError {
                            expected: ValueType::Int.into(),
                            got: other,
                            context: TypeErrorCtx::StringMul,
                        })
                    }
                }
                _ => Err(Error::TypeError {
                    expected: vec![ValueType::Num, ValueType::String],
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Num,
                        op: InfixOp::Mul,
                    },
                }),
            },
            Value::String(lhs) => match other.0 {
                Value::Num(rhs) => {
                    if rhs == rhs.trunc() {
                        Ok(Value::String(lhs.repeat(rhs as usize)))
                    } else {
                        Err(Error::TypeError {
                            expected: ValueType::Int.into(),
                            got: other,
                            context: TypeErrorCtx::StringMul,
                        })
                    }
                }
                _ => Err(Error::TypeError {
                    expected: ValueType::Num.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::String,
                        op: InfixOp::Mul,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: vec![ValueType::Num, ValueType::String],
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Mul },
            }),
        }
    }

    pub fn div(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Num(lhs) => match other.0 {
                Value::Num(rhs) => Ok(Value::Num(lhs / rhs)),
                _ => Err(Error::TypeError {
                    expected: ValueType::Num.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Num,
                        op: InfixOp::Div,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Num.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Div },
            }),
        }
    }

    pub fn modulus(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Num(lhs) => match other.0 {
                Value::Num(rhs) => Ok(Value::Num(lhs % rhs)),
                _ => Err(Error::TypeError {
                    expected: ValueType::Num.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Num,
                        op: InfixOp::Mod,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Num.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Mod },
            }),
        }
    }

    pub fn add(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Num(lhs) => match other.0 {
                Value::Num(rhs) => Ok(Value::Num(lhs + rhs)),
                _ => Err(Error::TypeError {
                    expected: ValueType::Num.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Num,
                        op: InfixOp::Add,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Num.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Add },
            }),
        }
    }

    pub fn sub(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Num(lhs) => match other.0 {
                Value::Num(rhs) => Ok(Value::Num(lhs - rhs)),
                _ => Err(Error::TypeError {
                    expected: ValueType::Num.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Num,
                        op: InfixOp::Sub,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Num.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Sub },
            }),
        }
    }

    pub fn lt(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Num(lhs) => match other.0 {
                Value::Num(rhs) => Ok(Value::Bool(lhs < rhs)),
                _ => Err(Error::TypeError {
                    expected: ValueType::Num.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Num,
                        op: InfixOp::Lt,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Num.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Lt },
            }),
        }
    }

    pub fn gt(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Num(lhs) => match other.0 {
                Value::Num(rhs) => Ok(Value::Bool(lhs > rhs)),
                _ => Err(Error::TypeError {
                    expected: ValueType::Num.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Num,
                        op: InfixOp::Gt,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Num.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Gt },
            }),
        }
    }

    pub fn lte(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Num(lhs) => match other.0 {
                Value::Num(rhs) => Ok(Value::Bool(lhs <= rhs)),
                _ => Err(Error::TypeError {
                    expected: ValueType::Num.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Num,
                        op: InfixOp::Lte,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Num.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Lte },
            }),
        }
    }

    pub fn gte(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Num(lhs) => match other.0 {
                Value::Num(rhs) => Ok(Value::Bool(lhs >= rhs)),
                _ => Err(Error::TypeError {
                    expected: ValueType::Num.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Num,
                        op: InfixOp::Gte,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Num.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Gte },
            }),
        }
    }

    pub fn and(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Bool(lhs) => match other.0 {
                Value::Bool(rhs) => Ok(Value::Bool(lhs && rhs)),
                _ => Err(Error::TypeError {
                    expected: ValueType::Bool.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Bool,
                        op: InfixOp::And,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Bool.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::And },
            }),
        }
    }

    pub fn or(self, other: Self) -> Result<Value, Error> {
        match self.0 {
            Value::Bool(lhs) => match other.0 {
                Value::Bool(rhs) => Ok(Value::Bool(lhs || rhs)),
                _ => Err(Error::TypeError {
                    expected: ValueType::Bool.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs: ValueType::Bool,
                        op: InfixOp::Or,
                    },
                }),
            },
            _ => Err(Error::TypeError {
                expected: ValueType::Bool.into(),
                got: self,
                context: TypeErrorCtx::InfixOpLhs { op: InfixOp::Or },
            }),
        }
    }

    pub fn equals(self, other: Self) -> Result<Value, Error> {
        match (self.0.clone(), other.0.clone()) {
            (Value::Num(lhs), Value::Num(rhs)) => Ok(Value::Bool(lhs == rhs)),
            (Value::String(lhs), Value::String(rhs)) => Ok(Value::Bool(lhs == rhs)),
            (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs == rhs)),
            (Value::Array(lhs), Value::Array(rhs)) => Ok(Value::Bool(lhs == rhs)),
            _ => {
                let lhs = self.0.get_type();

                Err(Error::TypeError {
                    expected: lhs.into(),
                    got: other,
                    context: TypeErrorCtx::InfixOpRhs {
                        lhs,
                        op: InfixOp::Equals,
                    },
                })
            }
        }
    }

    pub fn is_in(self, other: Self) -> Result<Value, Error> {
        todo!()
    }

    pub fn not(self) -> Result<Value, Error> {
        match self.0 {
            Value::Bool(e) => Ok(Value::Bool(!e)),
            _ => Err(Error::TypeError {
                expected: ValueType::Bool.into(),
                got: self,
                context: TypeErrorCtx::Not,
            }),
        }
    }

    pub fn index(self, idx: Self) -> Result<Value, Error> {
        match idx.0 {
            Value::Num(e) => {
                if e == e.trunc() {
                    let e = e as usize;

                    match self.0 {
                        Value::Array(f) => {
                            let len = f.len();

                            if len > e {
                                Ok(f[e].clone().0)
                            } else {
                                Err(Error::IndexError { index: e, len })
                            }
                        }
                        Value::String(f) => {
                            let len = f.len();

                            if len > e {
                                Ok(Value::String(f.chars().nth(e).unwrap().to_string()))
                            } else {
                                Err(Error::IndexError { index: e, len })
                            }
                        }
                        _ => Err(Error::TypeError {
                            expected: vec![ValueType::Array, ValueType::String],
                            got: self,
                            context: TypeErrorCtx::IndexOf,
                        }),
                    }
                } else {
                    Err(Error::TypeError {
                        expected: ValueType::Int.into(),
                        got: self,
                        context: TypeErrorCtx::Index,
                    })
                }
            }
            _ => Err(Error::TypeError {
                expected: ValueType::Num.into(),
                got: idx,
                context: TypeErrorCtx::Index,
            }),
        }
    }
}