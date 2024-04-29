use super::{Function, FunctionBuilder};
use crate::{
    expr::{Expr, StaticExpr},
    ScopeData,
};

pub fn sub() -> impl FunctionBuilder {
    |args| SubFunction { args }
}

pub struct SubFunction {
    args: Vec<Expr>,
}

impl Function for SubFunction {
    fn dependencies(&self) -> Vec<String> {
        self.args.iter().flat_map(|arg| arg.deps()).collect()
    }

    fn run(&self, scope: &ScopeData) -> f64 {
        let StaticExpr::Number(mut out) = self.args[0].run(scope) else {
            todo!()
        };

        for arg in &self.args[1..] {
            if let StaticExpr::Number(n) = arg.run(scope) {
                out -= n
            } else {
                todo!()
            }
        }

        out
    }
}
