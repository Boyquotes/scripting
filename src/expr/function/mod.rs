use super::Expr;
use crate::Scope;
use std::sync::Arc;

mod add;
pub use self::add::{add, AddFunction};

mod query;
pub use self::query::{query, QueryFunction};

pub trait FunctionBuilder: Send + Sync + 'static {
    type Function: Function;

    fn build(&self, args: Vec<Expr>) -> Self::Function;
}

impl<F, Func> FunctionBuilder for F
where
    F: Fn(Vec<Expr>) -> Func + Send + Sync + 'static,
    Func: Function,
{
    type Function = Func;

    fn build(&self, args: Vec<Expr>) -> Self::Function {
        self(args)
    }
}

pub trait Function: Send + Sync + 'static {
    fn dependencies(&self) -> Vec<String>;

    fn run(&self, scope: &Scope) -> f64;
}

pub(crate) trait DynFunctionBuilder: Send + Sync + 'static {
    fn dyn_build(&self, args: Vec<Expr>) -> Arc<dyn Function>;
}

impl<F: FunctionBuilder> DynFunctionBuilder for F {
    fn dyn_build(&self, args: Vec<Expr>) -> Arc<dyn Function> {
        Arc::new(self.build(args))
    }
}
