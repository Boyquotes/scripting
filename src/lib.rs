use std::{collections::HashMap, sync::Arc};

pub mod expr;
use self::expr::{
    function::{DynFunctionBuilder, FunctionBuilder},
    Expr, StaticExpr,
};

#[derive(Default)]
pub struct Registry {
    fns: HashMap<String, Arc<dyn DynFunctionBuilder>>,
}

impl Registry {
    pub fn insert(&mut self, id: impl Into<String>, builder: impl FunctionBuilder + 'static) {
        self.fns.insert(id.into(), Arc::new(builder));
    }
}
pub struct Scope {
    expr: Expr,
    dependencies: HashMap<String, Option<f64>>,
}

impl Scope {
    pub fn set_dependency(&mut self, id: &str, value: f64) {
        *self.dependencies.get_mut(id).unwrap() = Some(value);
    }

    pub fn run(&self) -> Option<StaticExpr> {
        if !self.dependencies.values().all(Option::is_some) {
            return None;
        }

        Some(self.expr.run(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{
        function::{AddFunctionBuilder, QueryFunctionBuilder},
        ExprData,
    };

    #[test]
    fn it_works() {
        let mut registry = Registry::default();
        registry.insert("+", AddFunctionBuilder);
        registry.insert("@", QueryFunctionBuilder);

        let data: ExprData = serde_json::from_str(r#" [ "+", [ "@", "test" ], 2 ] "#).unwrap();
        let mut scope = data.build(&registry);
        dbg!(scope.run());

        scope.set_dependency("test", 3.);
        dbg!(scope.run());
    }
}
