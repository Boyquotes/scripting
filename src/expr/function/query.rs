use super::{Function, FunctionBuilder};
use crate::{
    expr::{Expr, StaticExpr},
    ScopeData,
};

pub fn query() -> impl FunctionBuilder {
    |args: Vec<Expr>| {
        if let Some(Expr::Static(StaticExpr::String(s))) = args.first() {
            QueryFunction {
                dependency: s.clone(),
            }
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

    fn run(&self, scope: &ScopeData) -> f64 {
        scope.dependencies.get(&self.dependency).unwrap().unwrap()
    }
}
