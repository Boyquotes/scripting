use crate::{
    expr::{
        function::{self, FunctionBuilder},
        StaticExpr,
    },
    AssetRegistry, ComponentsData, Depends, LoadScript, Register, Registry, Scope, ScopeData,
    ScriptBundle, ScriptComponent, ScriptState, ScriptsReady,
};
use bevy::{
    app::{Plugin, Update},
    asset::{AssetServer, Assets},
    ecs::{
        component::Component,
        entity::Entity,
        event::{EventReader, EventWriter},
        query::{Changed, With},
        schedule::{NextState, State},
        system::{Commands, Query, Res, ResMut},
    },
    prelude::App,
};
use bevy_common_assets::json::JsonAssetPlugin;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

type SystemFn = Arc<dyn Fn(&mut App) + Send + Sync>;

pub struct ScriptPlugin {
    registry: Registry,
    add_system_fns: Vec<SystemFn>,
}

impl ScriptPlugin {
    pub fn empty() -> Self {
        Self {
            registry: Registry::default(),
            add_system_fns: Vec::new(),
        }
    }

    pub fn with_component<C: ScriptComponent + Default + DerefMut<Target = f64>>(
        mut self,
        id: impl Into<String>,
    ) -> Self {
        let id = id.into();

        self.registry.spawn_fns.insert(
            id.clone(),
            Arc::new(|value, registry, asset_server, entity_commands| {
                let data: C::Data = serde_json::from_value(value).unwrap();
                data.register::<C>(registry, asset_server, entity_commands);
            }),
        );
        self.registry.add_dependency::<C>(id);

        self.add_system_fns.push(Arc::new(|app: &mut App| {
            app.add_systems(Update, (run_lazy::<C>, run_expr::<C>));
        }));

        self
    }

    pub fn with_function(mut self, id: impl Into<String>, builder: impl FunctionBuilder) -> Self {
        self.registry.add_function(id, builder);
        self
    }
}

impl Default for ScriptPlugin {
    fn default() -> Self {
        Self::empty()
            .with_function("@", function::query())
            .with_function("+", function::add())
            .with_function("-", function::sub())
            .with_function("/", function::div())
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

#[allow(clippy::too_many_arguments)]
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

type ExprQuery<'w, 's, T> = Query<
    'w,
    's,
    (Entity, Option<&'static mut T>, &'static ScopeData),
    (With<Scope<T>>, Changed<ScopeData>),
>;

fn run_expr<T>(mut commands: Commands, mut query: ExprQuery<T>)
where
    T: Component + Default + DerefMut<Target = f64>,
{
    for (entity, value, scope_data) in &mut query {
        if let Some(StaticExpr::Number(new)) = scope_data.run() {
            if let Some(mut v) = value {
                if **v != new {
                    **v = new;
                }
            } else {
                let mut new_value = T::default();
                *new_value = new;

                commands.entity(entity).insert(new_value);
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
