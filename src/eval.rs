use std::collections::HashMap;

use crate::{
    ast::{Expr, InfixOp, Literal, Spanned},
    error::{Error, TypeErrorCtx},
    value::{SpannedValue, Value, ValueType},
};

#[derive(Clone, Debug)]
pub struct Output {
    pub value: SpannedValue,
    pub inputs: Vec<(String, ValueType)>,
    pub intermediates: HashMap<String, Expr>,
}

// returns either output and missing inputs, or an error
pub fn eval(
    input: &Spanned,
    vars: HashMap<String, Value>,
) -> Result<(SpannedValue, Vec<(String, ValueType)>), Vec<Error>> {
    let mut errors: Vec<Error> = Vec::new();

    match input {
        Spanned(Expr::Literal(literal), span) => match literal {
            Literal::Array(e) => {
                let mut errored = false;
                let mut inputs = Vec::new();

                let new = e
                    .iter()
                    .map(|f| {
                        let evaluated = eval(f, vars.clone());

                        match evaluated {
                            Ok(e) => {
                                inputs.extend(e.1);
                                e.0
                            }
                            Err(e) => {
                                errors.extend(e);
                                errored = true;

                                SpannedValue(Value::Error, f.clone().1)
                            }
                        }
                    })
                    .collect();

                if errored {
                    Err(errors)
                } else {
                    Ok((SpannedValue(Value::Array(new), span.clone()), inputs))
                }
            }
            _ => Ok((
                SpannedValue(Value::from(literal.clone()), span.clone()),
                Vec::new(),
            )),
        },
        Spanned(Expr::Assign { names, value }, span) => {
            let evaluated = eval(value, vars);

            match evaluated {
                Ok((spanned, inputs)) => Ok((
                    SpannedValue(
                        Value::Assign(names.clone(), Box::new(spanned.0)),
                        span.clone(),
                    ),
                    inputs,
                )),
                Err(error) => {
                    errors.extend(error);

                    Err(errors)
                }
            }
        }
        Spanned(Expr::InfixOp(lhs, op, rhs), span) => {
            let mut inputs = Vec::new();
            let lhs_span = lhs.1.clone();
            let rhs_span = rhs.1.clone();

            let lhs = eval(lhs, vars.clone());
            let lhs = match lhs {
                Ok(e) => {
                    inputs.extend(e.1);
                    e.0.clone()
                }
                Err(e) => {
                    errors.extend(e);

                    SpannedValue(Value::Error, lhs_span)
                }
            };

            let rhs = eval(rhs, vars);
            let rhs = match rhs {
                Ok((SpannedValue(Value::Input(name, kind, value), _), _)) => match *value {
                    Value::None => {
                        inputs.push((name, kind));
                        SpannedValue(Value::None, span.clone())
                    }
                    _ => SpannedValue(*value, span.clone()),
                },
                Ok(e) => {
                    inputs.extend(e.1);
                    e.0.clone()
                }
                Err(e) => {
                    errors.extend(e);

                    SpannedValue(Value::Error, rhs_span)
                }
            };

            if lhs == Value::Error || rhs == Value::Error {
                return Err(errors);
            }

            let output = match op {
                InfixOp::Pow => lhs.pow(rhs),
                InfixOp::Mul => lhs.mul(rhs),
                InfixOp::Div => lhs.div(rhs),
                InfixOp::Mod => lhs.modulus(rhs),
                InfixOp::Add => lhs.add(rhs),
                InfixOp::Sub => lhs.sub(rhs),
                InfixOp::Equals => lhs.equals(rhs),
                InfixOp::NotEquals => lhs.not_equals(rhs),
                InfixOp::Lt => lhs.lt(rhs),
                InfixOp::Gt => lhs.gt(rhs),
                InfixOp::Lte => lhs.lte(rhs),
                InfixOp::Gte => lhs.gte(rhs),
                InfixOp::And => lhs.and(rhs),
                InfixOp::Or => lhs.or(rhs),
                InfixOp::In => lhs.contains(rhs),
                InfixOp::Range => lhs.range(rhs),
                InfixOp::IRange => lhs.irange(rhs),
            };

            match output {
                Err(e) => {
                    errors.push(e);

                    Err(errors)
                }
                Ok(e) => Ok((SpannedValue(e, span.clone()), inputs)),
            }
        }
        Spanned(Expr::Not(rhs), span) => {
            let mut inputs = Vec::new();

            let rhs = eval(rhs, vars);
            let rhs = match rhs {
                Err(e) => {
                    errors.extend(e);

                    return Err(errors);
                }
                Ok(e) => {
                    inputs.extend(e.1);
                    e.0.clone()
                }
            };

            let output = rhs.not();

            match output {
                Err(e) => {
                    errors.push(e);

                    Err(errors)
                }
                Ok(e) => Ok((SpannedValue(e, span.clone()), inputs)),
            }
        }
        Spanned(Expr::Index(lhs, rhs), span) => {
            let mut inputs = Vec::new();

            let lhs = eval(lhs, vars.clone());
            let lhs = match lhs {
                Err(e) => {
                    errors.extend(e);

                    SpannedValue(Value::Error, 0..1)
                }
                Ok(e) => {
                    inputs.extend(e.1);
                    e.0.clone()
                }
            };

            let rhs = eval(rhs, vars);
            let rhs = match rhs {
                Err(e) => {
                    errors.extend(e);

                    SpannedValue(Value::Error, 0..1)
                }
                Ok(e) => {
                    inputs.extend(e.1);
                    e.0.clone()
                }
            };

            if lhs == Value::Error || rhs == Value::Error {
                return Err(errors);
            }

            let output = lhs.index(rhs);

            match output {
                Err(e) => {
                    errors.push(e);

                    Err(errors)
                }
                Ok(e) => Ok((SpannedValue(e, span.clone()), inputs)),
            }
        }
        Spanned(Expr::Ident(name), span) => match vars.get(name) {
            Some(out) => match out {
                Value::Input(name, kind, value) => match **value {
                    Value::None => Ok((
                        SpannedValue(
                            Value::Input(name.clone(), *kind, value.clone()),
                            span.clone(),
                        ),
                        vec![(name.clone(), *kind)],
                    )),
                    _ => Ok((SpannedValue(out.clone(), span.clone()), Vec::new())),
                },
                _ => Ok((SpannedValue(out.clone(), span.clone()), Vec::new())),
            },
            None => {
                let err = Error::ReferenceError {
                    name: name.clone(),
                    span: span.clone(),
                };
                errors.push(err);

                Err(errors)
            }
        },
        Spanned(
            Expr::Conditional {
                condition,
                inner,
                other,
            },
            span,
        ) => {
            let evaluated = eval(condition, vars.clone())?;

            dbg!(&evaluated);

            let out = match evaluated.0.clone() {
                SpannedValue(Value::Bool(enter), _) => {
                    if enter {
                        eval(inner, vars)
                    } else {
                        eval(other, vars)
                    }
                }
                SpannedValue(Value::Input(name, ValueType::Bool, value), _) => match *value {
                    Value::Bool(enter) => {
                        if enter {
                            eval(inner, vars)
                        } else {
                            eval(other, vars)
                        }
                    }
                    Value::None => Ok((
                        SpannedValue(Value::None, span.clone()),
                        vec![(name, ValueType::Bool)],
                    )),
                    _ => {
                        let err = Error::TypeError {
                            expected: ValueType::Bool.into(),
                            got: evaluated.0,
                            context: TypeErrorCtx::Condition,
                        };

                        errors.push(err);

                        return Err(errors);
                    }
                },
                _ => {
                    let err = Error::TypeError {
                        expected: ValueType::Bool.into(),
                        got: evaluated.0,
                        context: TypeErrorCtx::Condition,
                    };

                    errors.push(err);

                    return Err(errors);
                }
            };

            out
        }
        Spanned(Expr::Input(name, kind), span) => {
            dbg!(&name, &kind);

            Ok((
                SpannedValue(
                    Value::Input(name.clone(), *kind, Box::new(Value::None)),
                    span.clone(),
                ),
                Vec::new(),
            ))
        }
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chumsky::{Parser, Stream};

    use crate::{ast::Spanned, error::Error, lexer::lexer, parser, value::ValueType};

    use super::{SpannedValue, Value};

    fn parse<'a>(input: &'a str) -> Vec<Spanned> {
        let len = input.len();

        let lexed = lexer().parse(input).unwrap();
        parser::parse()
            .parse(Stream::from_iter(len..len + 1, lexed.into_iter()))
            .unwrap()
    }

    fn evaluate(input: &Spanned) -> Result<SpannedValue, Vec<Error>> {
        super::eval(input, HashMap::new()).map(|r| r.0)
    }

    #[test]
    fn evaluate_num() {
        let parsed = &parse("12")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Num(12.0))
    }

    #[test]
    fn evaluate_addition() {
        let parsed = &parse("12 + 8")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Num(20.0))
    }

    #[test]
    fn evaluate_chained() {
        let parsed = &parse("12 + 8 * 3")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Num(36.0))
    }

    #[test]
    fn evaluate_parens() {
        let parsed = &parse("(12 + 8) / 10")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Num(2.0))
    }

    #[test]
    fn evaluate_complex_chained() {
        let parsed = &parse("10 + (30 - 5) * 3 ** 2")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Num(235.0))
    }

    #[test]
    fn evaluate_string_mul() {
        let parsed = &parse("'nice' * 3")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::String("nicenicenice".to_owned()))
    }

    #[test]
    fn evaluate_string_mul_invalid_direct() {
        let parsed = &parse("'nice' * 'cool'")[0];
        let evaluated = evaluate(parsed);

        assert!(evaluated.is_err())
    }

    #[test]
    fn evaluate_string_mul_invalid_chain() {
        let parsed = &parse("'nice' * (3 * 'cool')")[0];
        let evaluated = evaluate(parsed);

        assert!(evaluated.is_err())
    }

    #[test]
    fn evaluate_index_array() {
        let parsed = &parse("[1, 2, 3, 4][3]")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Num(4.0))
    }

    #[test]
    fn evaluate_index_string() {
        let parsed = &parse("'nice'[3]")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::String("e".to_owned()))
    }

    #[test]
    fn evaluate_and_true() {
        let parsed = &parse("true and true")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Bool(true))
    }

    #[test]
    fn evaluate_and_false() {
        let parsed = &parse("true and false")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Bool(false))
    }

    #[test]
    fn evaluate_or_true() {
        let parsed = &parse("true or false")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Bool(true))
    }

    #[test]
    fn evaluate_or_false() {
        let parsed = &parse("false or false")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Bool(false))
    }

    #[test]
    fn evaluate_chain_and_or() {
        let parsed = &parse("true and false or true and true")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Bool(true))
    }

    #[test]
    fn evaluate_assign() {
        let parsed = &parse("nice = 'cool';")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(
            evaluated,
            Value::Assign(
                vec!["nice".to_owned()],
                Box::new(Value::String("cool".to_owned()))
            )
        )
    }

    #[test]
    fn evaluate_equals() {
        let parsed = &parse("60 == 60")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Bool(true))
    }

    #[test]
    fn evaluate_equals_false() {
        let parsed = &parse("50 == 60")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Bool(false))
    }

    #[test]
    fn evaluate_equals_fail() {
        let parsed = &parse("50 == [50]")[0];
        let evaluated = evaluate(parsed);

        assert!(evaluated.is_err())
    }

    #[test]
    fn evaluate_contains() {
        let parsed = &parse("12 in [10, 11, 12, 13, 14]")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Bool(true))
    }

    #[test]
    fn evaluate_index_array_fail() {
        let parsed = &parse("[1, 2, 3][3]")[0];
        let evaluated = evaluate(parsed);

        assert!(evaluated.is_err())
    }

    #[test]
    fn evaluate_index_string_fail() {
        let parsed = &parse(r#""nice"[10]"#)[0];
        let evaluated = evaluate(parsed);

        assert!(evaluated.is_err())
    }

    #[test]
    fn evaluate_assigned() {
        let parsed = parse(r#"cool = 23; nice = cool * 3;"#);
        let mut vars: HashMap<String, Value> = HashMap::new();

        let evaluated1 = evaluate(&parsed[0]).unwrap();

        match evaluated1 {
            SpannedValue(Value::Assign(names, value), _) => {
                for name in names {
                    vars.insert(name, *value.clone());
                }
            }
            _ => {
                assert!(false);
            }
        }

        let evaluated2 = super::eval(&parsed[1], vars).map(|r| r.0).unwrap();

        assert_eq!(
            evaluated2,
            Value::Assign(vec!["nice".to_owned()], Box::new(Value::Num(69.0)))
        )
    }

    #[test]
    fn evaluate_assign_chain() {
        let parsed = &parse("these = are = all = 12;")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(
            evaluated,
            Value::Assign(
                vec!["these".to_owned(), "are".to_owned(), "all".to_owned()],
                Box::new(Value::Num(12.0))
            )
        )
    }

    #[test]
    fn evaluate_range() {
        let parsed = &parse("0..5")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::Range(0..5))
    }

    #[test]
    fn evaluate_range_as_index() {
        let parsed = &parse("['nice', 'cool', 'wicked', 'sick'][1..3]")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(
            evaluated,
            Value::Array(vec![
                Value::String("cool".to_owned()).into(),
                Value::String("wicked".to_owned()).into()
            ])
        )
    }
    #[test]
    fn evaluate_irange_as_index() {
        let parsed = &parse("['nice', 'cool', 'wicked', 'sick'][1..=2]")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(
            evaluated,
            Value::Array(vec![
                Value::String("cool".to_owned()).into(),
                Value::String("wicked".to_owned()).into()
            ])
        )
    }

    #[test]
    fn evaluate_backwards_range_as_index() {
        let parsed = &parse("'wonderful'[-1..4]")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::String("lufr".to_owned()));
    }

    #[test]
    fn evaluate_backwards_irange_as_index() {
        let parsed = &parse("'sickening'[-4..=3]")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(evaluated, Value::String("nek".to_owned()));
    }

    #[test]
    fn evaluate_input() {
        let parsed = &parse("input cool;")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(
            evaluated,
            Value::Input("cool".to_owned(), ValueType::Any, Box::new(Value::None))
        )
    }

    #[test]
    fn evaluate_typed_input() {
        let parsed = &parse("input cool: Bool;")[0];
        let evaluated = evaluate(parsed).unwrap();

        assert_eq!(
            evaluated,
            Value::Input("cool".to_owned(), ValueType::Bool, Box::new(Value::None))
        )
    }
}
