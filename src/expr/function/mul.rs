use super::{Function, FunctionBuilder};
use crate::{
    expr::{Expr, StaticExpr},
    ScopeData,
};

pub fn mul() -> impl FunctionBuilder {
    |args| MulFunction { args }
}

pub struct MulFunction {
    args: Vec<Expr>,
}

impl Function for MulFunction {
    fn dependencies(&self) -> Vec<String> {
        self.args.iter().flat_map(|arg| arg.deps()).collect()
    }

    fn run(&self, scope: &ScopeData) -> f64 {
        let mut sum = 0.;

        for arg in &self.args {
            if let StaticExpr::Number(n) = arg.run(scope) {
                sum *= n
            } else {
                todo!()
            }
        }

        sum
    }
}
