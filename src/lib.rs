use bevy::{
    asset::{Asset, AssetServer, Handle},
    ecs::{
        component::Component,
        event::Event,
        schedule::States,
        system::{EntityCommands, Resource},
    },
    reflect::TypePath,
};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, marker::PhantomData, path::PathBuf, sync::Arc};

pub mod expr;
use self::expr::function::{DynFunctionBuilder, FunctionBuilder};

mod plugin;
pub use self::plugin::ScriptPlugin;

mod scope;
use self::scope::Dependency;
pub use scope::{Scope, ScopeData};

#[derive(Component)]
pub struct Depends<T> {
    id: String,
    _marker: PhantomData<T>,
}

pub trait ScriptComponent: Component {
    type Data: for<'de> Deserialize<'de> + Register;
}

pub trait Register {
    fn register<C: Component>(
        self,
        registry: &Registry,
        asset_server: &AssetServer,
        entity_commands: &mut EntityCommands,
    );
}

type SpawnFn = Arc<dyn Fn(Value, &Registry, &AssetServer, &mut EntityCommands) + Send + Sync>;

#[derive(Clone, Default, Resource)]
pub struct Registry {
    spawn_fns: HashMap<String, SpawnFn>,
    fns: HashMap<String, Arc<dyn DynFunctionBuilder>>,
    deps: HashMap<String, Arc<dyn Dependency>>,
    operations: HashMap<String, Arc<dyn Operation>>,
}

impl Registry {
    pub fn add_function(&mut self, id: impl Into<String>, builder: impl FunctionBuilder) {
        self.fns.insert(id.into(), Arc::new(builder));
    }

    pub fn add_dependency<C: Component>(&mut self, id: impl Into<String>) {
        self.deps.insert(id.into(), Arc::new(PhantomData::<C>));
    }

    pub fn spawn(
        &self,
        asset_server: &AssetServer,
        entity_commands: &mut EntityCommands,
        values: HashMap<String, Value>,
    ) {
        for (name, value) in values {
            self.spawn_fns.get(&name).unwrap()(value, self, asset_server, entity_commands)
        }
    }
}

#[derive(Component)]
pub struct ScriptBundle(pub String);

impl ScriptBundle {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

#[derive(Default, Resource)]
pub struct AssetRegistry {
    pub pending_handles: HashMap<String, Handle<ComponentsData>>,
    pub handles: HashMap<String, Handle<ComponentsData>>,
}

#[derive(Clone, Deserialize, Asset, TypePath)]
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

pub trait Operation: Send + Sync {
    fn spawn(
        &self,
        registry: &Registry,
        asset_server: &AssetServer,
        entity_commands: &mut EntityCommands,
        value: Value,
    );
}

pub struct AddOperation;

impl Operation for AddOperation {
    fn spawn(
        &self,
        registry: &Registry,
        asset_server: &AssetServer,
        entity_commands: &mut EntityCommands,
        value: Value,
    ) {
        let data: AddOperationData = serde_json::from_value(value).unwrap();
        match data {
            AddOperationData::Single(id) => {
                let f = registry.spawn_fns.get(&id).unwrap();
                f(Value::default(), registry, asset_server, entity_commands);
            }
            AddOperationData::Many(_) => todo!(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum AddOperationData {
    Single(String),
    Many(Vec<String>),
}

#[derive(Component)]
pub struct EventMarker<T> {
    pub _marker: PhantomData<T>,
}
