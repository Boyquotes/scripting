use bevy::{
    app::{Plugin, Update},
    prelude::App,
};
use bevy_common_assets::json::JsonAssetPlugin;

use crate::{
    expr::function::{self, FunctionBuilder},
    load_assets, run_expr, run_lazy, spawn_expr, AssetRegistry, ComponentsData, LoadScript,
    Register, Registry, ScriptComponent, ScriptState, ScriptsReady,
};


use std::{
    ops::{DerefMut},
    sync::Arc,
};

pub struct ScriptPlugin {
    registry: Registry,
    add_system_fns: Vec<Arc<dyn Fn(&mut App) + Send + Sync>>,
}

impl ScriptPlugin {
    pub fn empty() -> Self {
        Self {
            registry: Registry::default(),
            add_system_fns: Vec::new(),
        }
    }

    pub fn with_component<C: ScriptComponent>(mut self, id: impl Into<String>) -> Self {
        let id = id.into();

        self.registry.spawn_fns.insert(
            id.clone(),
            Arc::new(|value, registry, asset_server, entity_commands| {
                let data: C::Data = serde_json::from_value(value).unwrap();
                data.register::<C>(registry, asset_server, entity_commands);
            }),
        );
        self.registry.add_dependency::<C>(id);

        self
    }

    pub fn with_derived<C: ScriptComponent + Default + DerefMut<Target = f64>>(
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
            .with_function("+", function::add())
            .with_function("@", function::query())
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
