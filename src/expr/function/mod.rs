use std::sync::Arc;

use crate::Scope;

use super::Expr;

mod add;
pub use self::add::{AddFunction, AddFunctionBuilder};

mod query;
pub use self::query::{QueryFunction, QueryFunctionBuilder};

pub trait FunctionBuilder {
    fn build(&self, args: Vec<Expr>) -> Arc<dyn Function>;
}

pub trait Function {
    fn dependencies(&self) -> Vec<String>;

    fn run(&self, scope: &Scope) -> f64;
}
