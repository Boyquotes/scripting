use bevy::{
    app::{Plugin, Update},
    asset::{Asset, AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::{Changed, With},
        schedule::{NextState, State, States},
        system::{Commands, EntityCommands, Query, Res, ResMut, Resource},
    },
    prelude::App,
    reflect::TypePath,
};
use bevy_common_assets::json::JsonAssetPlugin;

use scope::Dependency;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
};

pub mod expr;
use self::expr::{
    function::{DynFunctionBuilder, FunctionBuilder},
    StaticExpr,
};

mod scope;
pub use scope::{Scope, ScopeData};

#[derive(Component)]
pub struct Depends<T> {
    id: String,
    _marker: PhantomData<T>,
}

pub trait DynamicComponent: Component {
    type Data: for<'de> Deserialize<'de>;

    fn register(data: Self::Data, registry: &Registry, entity_commands: &mut EntityCommands);
}

#[derive(Clone, Default, Resource)]
pub struct Registry {
    spawn_fns: HashMap<String, Arc<dyn Fn(Value, &Self, &mut EntityCommands) + Send + Sync>>,
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

    pub fn spawn(&self, entity_commands: &mut EntityCommands, values: HashMap<String, Value>) {
        for (name, value) in values {
            self.spawn_fns.get(&name).unwrap()(value, self, entity_commands)
        }
    }
}

#[derive(Default)]
pub struct ScriptPlugin {
    registry: Registry,
    add_system_fns: Vec<Arc<dyn Fn(&mut App) + Send + Sync>>,
}

impl ScriptPlugin {
    pub fn with_bundle<T: DynamicComponent>(mut self, id: impl Into<String>) -> Self {
        self.registry.spawn_fns.insert(
            id.into(),
            Arc::new(|value, registry, entity_commands| {
                let data: T::Data = serde_json::from_value(value).unwrap();
                T::register(data, registry, entity_commands);
            }),
        );
        self
    }

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
        app.add_plugins(JsonAssetPlugin::<ComponentsData>::new(&[]))
            .insert_resource(self.registry.clone())
            .init_resource::<AssetRegistry>()
            .insert_state(ScriptState::Ready)
            .add_event::<LoadScript>()
            .add_event::<ScriptsReady>()
            .add_systems(Update, (load_assets, spawn_expr));

        for f in &self.add_system_fns {
            f(app)
        }
    }
}

#[derive(Component)]
pub struct ScriptBundle(pub String);

#[derive(Default, Resource)]
pub struct AssetRegistry {
    pub handles: HashMap<String, Handle<ComponentsData>>,
}

fn load_assets(
    mut asset_registry: ResMut<AssetRegistry>,
    asset_server: Res<AssetServer>,
    mut asset_events: EventReader<LoadScript>,
    mut state: ResMut<NextState<ScriptState>>,
) {
    for event in asset_events.read() {
        let handle = asset_server.load(event.path.clone());
        asset_registry
            .handles
            .insert(String::from("sword.json"), handle);

        state.set(ScriptState::Loading);
    }
}

fn spawn_expr(
    mut commands: Commands,
    asset_registry: Res<AssetRegistry>,
    assets: Res<Assets<ComponentsData>>,
    registry: Res<Registry>,
    query: Query<(Entity, &ScriptBundle)>,
    mut asset_events: EventWriter<ScriptsReady>,
    state: Res<State<ScriptState>>,
    mut next_state: ResMut<NextState<ScriptState>>,
) {
    let mut is_ready = true;

    for (entity, bundle) in &query {
        let handle = asset_registry.handles.get(&bundle.0).unwrap();
        if let Some(data) = assets.get(handle) {
            registry.spawn(&mut commands.entity(entity), data.0.clone());

            commands.entity(entity).remove::<ScriptBundle>();
        } else {
            is_ready = false;
        }
    }

    if is_ready {
        if *state == ScriptState::Loading {
            next_state.set(ScriptState::Ready);
            asset_events.send(ScriptsReady);
        }
    }
}

fn run_expr<T>(mut query: Query<(&mut T, &ScopeData), (With<Scope<T>>, Changed<ScopeData>)>)
where
    T: Component + Default + DerefMut<Target = f64>,
{
    for (mut value, scope_data) in &mut query {
        if let Some(StaticExpr::Number(new)) = scope_data.run() {
            if **value != new {
                let mut new_value = T::default();
                *new_value = new;

                *value = new_value;
            }
        }
    }
}

fn run_lazy<T>(mut query: Query<(&mut ScopeData, &T, &Depends<T>)>)
where
    T: Component + Deref<Target = f64>,
{
    for (mut scope_data, value, dep) in &mut query {
        scope_data.set_dependency(&dep.id, **value);
    }
}

#[derive(Deserialize, Asset, TypePath)]
pub struct ComponentsData(pub HashMap<String, Value>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, States)]
pub enum ScriptState {
    Loading,
    Ready,
}

#[derive(Event)]
pub struct LoadScript {
    path: PathBuf,
}

impl LoadScript {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[derive(Event)]
pub struct ScriptsReady;
