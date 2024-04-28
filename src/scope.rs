use super::expr::{Expr, StaticExpr};
use crate::{Depends, Registry};
use bevy::ecs::{component::Component, system::EntityCommands};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component)]
pub struct Scope<T> {
    _marker: PhantomData<T>,
}

#[derive(Component)]
pub struct ScopeData {
    pub(crate) expr: Expr,
    pub(crate) dependencies: HashMap<String, Option<f64>>,
}

impl ScopeData {
    pub fn spawn<T: Component>(self, registry: &Registry, entity_commands: &mut EntityCommands) {
        for id in self.dependencies.keys() {
            let dep = registry.deps.get(id).unwrap();
            dep.spawn(id.clone(), entity_commands);
        }

        entity_commands.insert((
            self,
            Scope {
                _marker: PhantomData::<T>,
            },
        ));
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

pub(crate) trait Dependency: Send + Sync + 'static {
    fn spawn(&self, id: String, entity_commands: &mut EntityCommands);
}

impl<C: Component> Dependency for PhantomData<C> {
    fn spawn(&self, id: String, entity_commands: &mut EntityCommands) {
        entity_commands.insert(Depends {
            id,
            _marker: PhantomData::<C>,
        });
    }
}
