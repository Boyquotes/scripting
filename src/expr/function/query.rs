use super::{Function, FunctionBuilder};
use crate::{
    expr::{Expr, StaticExpr},
    Scope,
};
use std::sync::Arc;

pub struct QueryFunctionBuilder;

impl FunctionBuilder for QueryFunctionBuilder {
    fn build(&self, args: Vec<Expr>) -> Arc<dyn Function> {
        if let Some(Expr::Static(StaticExpr::String(s))) = args.first() {
            Arc::new(QueryFunction {
                dependency: s.clone(),
            })
        } else {
            todo!()
        }
    }
}

pub struct QueryFunction {
    dependency: String,
}

impl Function for QueryFunction {
    fn dependencies(&self) -> Vec<String> {
        vec![self.dependency.clone()]
    }

    fn run(&self, scope: &Scope) -> f64 {
        scope.dependencies.get(&self.dependency).unwrap().unwrap()
    }
}
