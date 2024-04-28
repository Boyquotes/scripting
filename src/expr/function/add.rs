use super::{Function, FunctionBuilder};
use crate::{
    expr::{Expr, StaticExpr},
    Scope,
};

pub struct AddFunctionBuilder;

impl FunctionBuilder for AddFunctionBuilder {
    type Function = AddFunction;

    fn build(&self, args: Vec<Expr>) -> Self::Function {
        AddFunction { args }
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
