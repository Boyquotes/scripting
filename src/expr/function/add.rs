use super::{Function, FunctionBuilder};
use crate::{
    expr::{Expr, StaticExpr},
    ScopeData,
};

pub fn add() -> impl FunctionBuilder {
    |args| AddFunction { args }
}

pub struct AddFunction {
    args: Vec<Expr>,
}

impl Function for AddFunction {
    fn dependencies(&self) -> Vec<String> {
        self.args.iter().map(|arg| arg.deps()).flatten().collect()
    }

    fn run(&self, scope: &ScopeData) -> f64 {
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
