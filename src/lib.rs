use bevy::ecs::{
    component::Component,
    system::{EntityCommands, Resource},
};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};

pub mod expr;
use self::expr::{
    function::{DynFunctionBuilder, FunctionBuilder},
    Expr, StaticExpr,
};

#[derive(Component)]
pub struct Depends<T> {
    _marker: PhantomData<T>,
}

trait Dependency: Send + Sync + 'static {
    fn spawn(&self, _entity_commands: &mut EntityCommands) {}
}

impl<C: Component> Dependency for PhantomData<C> {
    fn spawn(&self, entity_commands: &mut EntityCommands) {
        entity_commands.insert(Depends {
            _marker: PhantomData::<Self>,
        });
    }
}

#[derive(Component)]
pub struct Health(f64);

#[derive(Default, Resource)]
pub struct Registry {
    fns: HashMap<String, Arc<dyn DynFunctionBuilder>>,
    deps: HashMap<String, Arc<dyn Dependency>>,
}

impl Registry {
    pub fn add_function(&mut self, id: impl Into<String>, builder: impl FunctionBuilder) {
        self.fns.insert(id.into(), Arc::new(builder));
    }

    pub fn add_dependency<C: Component>(&mut self, id: impl Into<String>) {
        self.deps.insert(id.into(), Arc::new(PhantomData::<C>));
    }
}

#[derive(Component)]
pub struct Scope {
    expr: Expr,
    dependencies: HashMap<String, Option<f64>>,
}

impl Scope {
    pub fn spawn(self, registry: &Registry, entity_commands: &mut EntityCommands) {
        for id in self.dependencies.keys() {
            let dep = registry.deps.get(id).unwrap();
            dep.spawn(entity_commands);
        }

        entity_commands.insert(self);
    }

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

        registry.add_dependency::<Health>("health");

        registry.add_function("+", AddFunctionBuilder);
        registry.add_function("@", QueryFunctionBuilder);

        let data: ExprData = serde_json::from_str(r#" [ "+", [ "@", "health" ], 2 ] "#).unwrap();
        let mut scope = data.build(&registry);
        dbg!(scope.run());

        scope.set_dependency("health", 3.);
        dbg!(scope.run());
    }
}
