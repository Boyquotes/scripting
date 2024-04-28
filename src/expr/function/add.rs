use super::{Function, FunctionBuilder};
use crate::{
    expr::{Expr, StaticExpr},
    Scope,
};
use std::sync::Arc;

pub struct AddFunctionBuilder;

impl FunctionBuilder for AddFunctionBuilder {
    fn build(&self, args: Vec<Expr>) -> Arc<dyn Function> {
        Arc::new(AddFunction { args })
    }
}

pub struct AddFunction {
    args: Vec<Expr>,
}

impl Function for AddFunction {
    fn dependencies(&self) -> Vec<String> {
        self.args.iter().map(|arg| arg.deps()).flatten().collect()
    }

    fn run(&self, scope: &Scope) -> f64 {
        let mut sum = 0.;

        for arg in &self.args {
            if let StaticExpr::Number(n) = arg.run(scope) {
                sum += n
            } else {
                todo!()
            }
        }

        sum
    }
}
