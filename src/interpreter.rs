use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
};

use chumsky::{Parser, Stream};

use crate::{
    ast::{Expr, Span, Spanned},
    error::Error,
    eval::eval,
    lexer, parser,
    value::{Value, ValueType},
};

#[derive(Clone, Debug)]
pub struct SpannedIdent {
    name: String,
    span: Span,
}

impl PartialEq<SpannedIdent> for SpannedIdent {
    fn eq(&self, other: &SpannedIdent) -> bool {
        self.name == other.name
    }
}

impl Eq for SpannedIdent {}

impl From<String> for SpannedIdent {
    fn from(f: String) -> Self {
        Self {
            name: f,
            span: 0..1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VarStore {
    pub values: HashMap<String, Value>,            // cached values
    pub(crate) inputs: Vec<(String, ValueType)>,   // all inputs with their types
    pub(crate) deps: HashMap<String, Vec<String>>, // variables that each variable depends on
    pub(crate) dependents: HashMap<String, Vec<String>>, // inverse of deps, variables that depend on each variable
    pub(crate) source: HashMap<String, String>, // generated source for each variable, so it can be serialized easier
    pub(crate) cached: HashMap<String, bool>,   // whether the cached value for a variable is valid
    pub(crate) intermediate: HashMap<String, HashMap<String, Expr>>,
}

pub fn interpret(input: &str) -> Result<VarStore, Vec<Error>> {
    let len = input.len();

    let (lexed, errs) = lexer::lexer().parse_recovery(input);

    if errs.len() > 0 {
        return Err(errs.iter().map(|e| Error::SyntaxError(e.clone())).collect());
    }

    let (parsed, errs) =
        parser::parse().parse_recovery(Stream::from_iter(len..len + 1, lexed.unwrap().into_iter()));

    if errs.len() > 0 {
        return Err(errs
            .iter()
            .map(|e| Error::ParsingError(e.clone()))
            .collect());
    }

    let parsed = parsed.unwrap();

    let mut spans: HashMap<String, Span> = HashMap::new();
    let mut errs: Vec<Error> = Vec::new();
    let mut deps: HashMap<String, (Vec<String>, Span)> = HashMap::new();
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
    let mut out_deps: HashMap<String, Vec<String>> = HashMap::new();
    let mut cached: HashMap<String, bool> = HashMap::new();

    // check dependencies of variables
    for expr in parsed.iter() {
        match expr {
            Spanned(Expr::Assign { names, value }, span) => {
                let value_deps = get_deps(value);

                for name in names {
                    if let Some(old_span) = spans.get(name) {
                        let err = Error::ReassignError {
                            name: name.to_string(),
                            old_span: old_span.clone(),
                            new_span: span.clone(),
                        };
                        errs.push(err);
                    } else {
                        spans.insert(name.clone(), span.clone());
                        deps.insert(name.to_owned(), (value_deps.clone(), span.clone()));
                        out_deps.insert(name.to_owned(), value_deps.clone());
                        cached.insert(name.to_owned(), true);

                        for dep in value_deps.clone() {
                            if let Some(item) = dependents.get_mut(&dep) {
                                item.push(name.to_owned());
                            } else {
                                let item = vec![name.to_owned()];

                                dependents.insert(dep, item);
                            }
                        }
                    }
                }
            }
            Spanned(Expr::Input(name, _), span) => {
                if let Some(old_span) = spans.get(name) {
                    let err = Error::ReassignError {
                        name: name.to_string(),
                        old_span: old_span.clone(),
                        new_span: span.clone(),
                    };
                    errs.push(err);
                } else {
                    spans.insert(name.clone(), span.clone());
                    deps.insert(name.to_owned(), (Vec::new(), span.clone()));
                    out_deps.insert(name.to_owned(), Vec::new());
                    cached.insert(name.to_owned(), false);
                }
            }
            _ => {}
        }
    }

    if errs.len() > 0 {
        return Err(errs);
    }

    let mut order: VecDeque<SpannedIdent> = VecDeque::new();

    // set an order to evaluate variables in
    while deps.len() > 0 {
        let mut changed = false;

        'outer: for (name, (inner_deps, span)) in deps.clone() {
            if inner_deps.len() == 0 {
                changed = true;
                deps.remove(&name);
                order.push_back(SpannedIdent { name, span });
            } else {
                for dep in inner_deps {
                    if !order.contains(&SpannedIdent::from(dep)) {
                        continue 'outer;
                    }
                }

                changed = true;
                deps.remove(&name);
                order.push_back(SpannedIdent {
                    name: name.clone(),
                    span,
                });
            }
        }

        if !changed {
            let mut chain: Vec<SpannedIdent> = Vec::new();
            let cloned = deps.clone();
            let (first_key, first) = cloned.iter().next().unwrap();

            return Err(gather_deps_errors(
                first_key.clone(),
                &mut chain,
                &mut deps,
                &first.1,
            ));
        }
    }

    let mut exprs: HashMap<String, Spanned> = HashMap::new();

    // gather the variable assignments without evaluating them
    for expr in parsed.iter() {
        match expr {
            Spanned(Expr::Assign { names, value }, _) => {
                for name in names {
                    exprs.insert(name.clone(), *value.clone());
                }
            }
            Spanned(Expr::Input(name, _), _) => {
                exprs.insert(name.clone(), expr.clone());
            }
            _ => {}
        }
    }

    let mut vars: HashMap<String, Value> = HashMap::new();
    let mut source: HashMap<String, String> = HashMap::new();
    let mut inputs: Vec<(String, ValueType)> = Vec::new();

    // finally, evaluate the variables
    for SpannedIdent { name, span: _ } in order {
        // into value
        let evaluated = eval(exprs.get(&name).unwrap(), vars.clone());
        // into source
        let expr_source = exprs.get(&name).unwrap().clone().into();
        source.insert(name.clone(), expr_source);

        match evaluated {
            Ok((value, inputs_out)) => {
                inputs.extend(inputs_out);
                vars.insert(name, value.0);
            }
            Err(e) => {
                errs.extend(e);
            }
        }
    }

    if errs.len() > 0 {
        Err(errs)
    } else {
        Ok(VarStore {
            values: vars,
            inputs,
            deps: out_deps,
            dependents,
            source,
            cached,
            intermediate: HashMap::new(),
        })
    }
}

fn get_deps(expr: &Spanned) -> Vec<String> {
    let mut deps: Vec<String> = Vec::new();

    match expr {
        Spanned(Expr::Ident(name), _) => {
            deps.push(name.clone());
        }
        Spanned(Expr::InfixOp(lhs, _, rhs), _) => {
            deps.extend(get_deps(lhs));
            deps.extend(get_deps(rhs));
        }
        Spanned(Expr::Index(lhs, idx), _) => {
            deps.extend(get_deps(lhs));
            deps.extend(get_deps(idx));
        }
        Spanned(Expr::Not(rhs), _) => {
            deps.extend(get_deps(rhs));
        }
        Spanned(Expr::Literal(_), _) => {}
        Spanned(Expr::Assign { names: _, value: _ }, _) => {
            unreachable!("Assigns can never be in the value of an assignment")
        }
        Spanned(Expr::Error, _) => {}
        Spanned(
            Expr::Conditional {
                condition,
                inner,
                other,
            },
            _,
        ) => {
            deps.extend(get_deps(condition));
            deps.extend(get_deps(inner));
            deps.extend(get_deps(other));
        }
        _ => todo!(),
    }

    deps
}

pub fn get_inputs(input: &str) -> Result<Vec<(String, ValueType)>, Vec<Error>> {
    let len = input.len();

    let (lexed, errs) = lexer::lexer().parse_recovery(input);

    if errs.len() > 0 {
        return Err(errs.iter().map(|e| Error::SyntaxError(e.clone())).collect());
    }

    let (parsed, errs) =
        parser::parse().parse_recovery(Stream::from_iter(len..len + 1, lexed.unwrap().into_iter()));

    if errs.len() > 0 {
        return Err(errs
            .iter()
            .map(|e| Error::ParsingError(e.clone()))
            .collect());
    }

    let parsed = parsed.unwrap();
    let mut inputs: Vec<(String, ValueType)> = Vec::new();

    for item in parsed {
        match item {
            Spanned(Expr::Input(name, kind), _) => inputs.push((name, kind)),
            _ => {}
        }
    }

    Ok(inputs)
}

fn gather_deps_errors(
    name: String,
    chain: &mut Vec<SpannedIdent>,
    deps: &mut HashMap<String, (Vec<String>, Span)>,
    parent_span: &Span,
) -> Vec<Error> {
    let mut errs: Vec<Error> = Vec::new();

    if chain.contains(&SpannedIdent::from(name.clone())) {
        let err = Error::RecursionError {
            chain: chain.clone(),
        };

        errs.push(err);
    } else if let Some((next_deps, span)) = deps.clone().get(&name) {
        chain.push(SpannedIdent {
            name,
            span: span.clone(),
        });

        for dep in next_deps {
            errs.extend(gather_deps_errors(dep.clone(), chain, deps, parent_span))
        }
    } else {
        // name is not in deps, so it has to be undefined
        let err = Error::ReferenceError {
            name,
            span: parent_span.clone(),
        };

        errs.push(err);
    }

    errs
}

#[cfg(test)]
mod tests {
    use crate::value::Value;

    use super::interpret;

    #[test]
    fn interpret_assign_chain() {
        let interpreted = interpret("these = are = all = 12;").unwrap().values;
        let value = Value::Num(12.0);

        assert_eq!(interpreted.get("these").unwrap(), &value);
        assert_eq!(interpreted.get("are").unwrap(), &value);
        assert_eq!(interpreted.get("all").unwrap(), &value)
    }
}
