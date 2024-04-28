use super::Expr;
use crate::Scope;
use std::sync::Arc;

mod add;
pub use self::add::{AddFunction, AddFunctionBuilder};

mod query;
pub use self::query::{QueryFunction, QueryFunctionBuilder};

pub trait FunctionBuilder {
    type Function: Function;

    fn build(&self, args: Vec<Expr>) -> Self::Function;
}

pub trait Function: 'static {
    fn dependencies(&self) -> Vec<String>;

    fn run(&self, scope: &Scope) -> f64;
}

pub(crate) trait DynFunctionBuilder {
    fn dyn_build(&self, args: Vec<Expr>) -> Arc<dyn Function>;
}

impl<F: FunctionBuilder> DynFunctionBuilder for F {
    fn dyn_build(&self, args: Vec<Expr>) -> Arc<dyn Function> {
        Arc::new(self.build(args))
    }
}
