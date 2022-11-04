mod ast;
pub mod errors;
pub mod eval;
pub mod interpreter;
pub mod operators;
pub mod parser;

use std::collections::HashMap;

use interpreter::Value;
use parser::parse;
