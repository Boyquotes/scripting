use bevy::{
    app::{Plugin, Update},
    ecs::{
        component::Component,
        query::Changed,
        system::{Query, Resource},
    },
    prelude::App,
};
use bevy_common_assets::json::JsonAssetPlugin;
use expr::ExprData;
use scope::Dependency;
use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub mod expr;
use self::expr::{
    function::{DynFunctionBuilder, FunctionBuilder},
    StaticExpr,
};

mod scope;
pub use scope::Scope;

#[derive(Component)]
pub struct Depends<T> {
    id: String,
    _marker: PhantomData<T>,
}

#[derive(Clone, Default, Resource)]
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

#[derive(Default)]
pub struct ScriptPlugin {
    registry: Registry,
    add_system_fns: Vec<Arc<dyn Fn(&mut App) + Send + Sync>>,
}

impl ScriptPlugin {
    pub fn with_dependency<C>(mut self, id: impl Into<String>) -> Self
    where
        C: Component + Default + DerefMut<Target = f64>,
    {
        self.registry.add_dependency::<C>(id);

        self.add_system_fns.push(Arc::new(|app: &mut App| {
            app.add_systems(Update, run_lazy::<C>);
        }));

        self.add_system_fns.push(Arc::new(|app: &mut App| {
            app.add_systems(Update, run_expr::<C>);
        }));

        self
    }

    pub fn with_function(mut self, id: impl Into<String>, builder: impl FunctionBuilder) -> Self {
        self.registry.add_function(id, builder);
        self
    }
}

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<ExprData>::new(&[]))
            .insert_resource(self.registry.clone());

        for f in &self.add_system_fns {
            f(app)
        }
    }
}

fn run_expr<T: Component + Default + DerefMut<Target = f64>>(
    mut query: Query<(&mut T, &Scope), Changed<Scope>>,
) {
    for (mut value, expr) in &mut query {
        if let Some(StaticExpr::Number(new)) = expr.run() {
            if **value != new {
                let mut new_value = T::default();
                *new_value = new;

                *value = new_value;
            }
        }
    }
}

fn run_lazy<T: Component + Deref<Target = f64>>(
    mut query: Query<(&mut Scope, &T, &Depends<T>), Changed<T>>,
) {
    for (mut scope, value, dep) in &mut query {
        scope.set_dependency(&dep.id, **value);
    }
}
