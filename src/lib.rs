use bevy::{
    asset::{Asset, AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::{Changed, With},
        schedule::{NextState, State, States},
        system::{Commands, EntityCommands, Query, Res, ResMut, Resource},
    },
    reflect::TypePath,
};


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

mod plugin;
pub use self::plugin::ScriptPlugin;

mod scope;
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

#[derive(Clone, Default, Resource)]
pub struct Registry {
    spawn_fns:
        HashMap<String, Arc<dyn Fn(Value, &Self, &AssetServer, &mut EntityCommands) + Send + Sync>>,
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

fn load_assets(
    mut asset_registry: ResMut<AssetRegistry>,
    asset_server: Res<AssetServer>,
    mut asset_events: EventReader<LoadScript>,
    mut state: ResMut<NextState<ScriptState>>,
) {
    for event in asset_events.read() {
        let handle = asset_server.load(event.path.clone());

        // TODO path or string?
        asset_registry
            .pending_handles
            .insert(event.path.to_string_lossy().to_string(), handle);

        state.set(ScriptState::Loading);
    }
}

fn spawn_expr(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut asset_registry: ResMut<AssetRegistry>,
    mut assets: ResMut<Assets<ComponentsData>>,
    registry: Res<Registry>,
    query: Query<(Entity, &ScriptBundle)>,
    mut asset_events: EventWriter<ScriptsReady>,
    state: Res<State<ScriptState>>,
    mut next_state: ResMut<NextState<ScriptState>>,
) {
    let mut ready_handles = Vec::new();
    for (path, handle) in &asset_registry.pending_handles {
        if let Some(data) = assets.get_mut(handle) {
            let id = data.0.remove("id").unwrap();
            let id_s = id.as_str().unwrap().to_owned();

            ready_handles.push((path.clone(), id_s, handle.clone()));
        }
    }

    for (path, id, handle) in ready_handles {
        asset_registry.pending_handles.remove(&path);
        asset_registry.handles.insert(id, handle);
    }

    let mut is_ready = true;
    for (entity, bundle) in &query {
        if let Some(handle) = asset_registry.handles.get(&bundle.0) {
            if let Some(data) = assets.get(handle) {
                registry.spawn(&asset_server, &mut commands.entity(entity), data.0.clone());

                commands.entity(entity).remove::<ScriptBundle>();
            } else {
                is_ready = false;
            }
        }
    }

    if is_ready && *state == ScriptState::Loading {
        next_state.set(ScriptState::Ready);
        asset_events.send(ScriptsReady);
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
