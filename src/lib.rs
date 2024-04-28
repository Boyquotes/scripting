use bevy::{
    app::{Plugin, Update},
    asset::Assets,
    ecs::{
        component::Component,
        query::{Changed},
        system::{Commands, EntityCommands, Query, Res, ResMut, Resource},
    },
    prelude::App,
};
use bevy_common_assets::json::JsonAssetPlugin;
use expr::ExprData;
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

#[derive(Default)]
pub struct ScriptPlugin {
    registry: Registry,
}

impl ScriptPlugin {
    pub fn with_dependency<C>(mut self, id: impl Into<String>) -> Self
    where
        C: Component,
    {
        self.registry.add_dependency::<C>(id);
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
            .insert_resource(self.registry.clone())
            .add_systems(Update, (spawn_expr, run_expr));
    }
}

fn spawn_expr(
    mut commands: Commands,
    mut expr_data_assets: ResMut<Assets<ExprData>>,
    registry: Res<Registry>,
) {
    let mut asset_ids = Vec::new();

    for (asset_id, expr_data) in expr_data_assets.iter_mut() {
        let mut entity_commands = commands.spawn_empty();
        expr_data
            .clone()
            .build(&registry)
            .spawn(&registry, &mut entity_commands);

        asset_ids.push(asset_id);
    }

    for id in asset_ids {
        expr_data_assets.remove(id);
    }
}

fn run_expr(query: Query<&Scope, Changed<Scope>>) {
    for expr in &query {
        dbg!("run!");
        expr.run();
    }
}
