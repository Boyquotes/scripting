use bevy::prelude::*;
use scripting::{
    expr::{
        function::{AddFunctionBuilder, QueryFunctionBuilder},
        ExprData,
    },
    Health, ScriptPlugin,
};

#[derive(Resource)]
struct SwordHandle(Handle<ExprData>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("sword.json");
    commands.insert_resource(SwordHandle(handle));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ScriptPlugin::default()
                .with_dependency::<Health>("health")
                .with_function("+", AddFunctionBuilder)
                .with_function("@", QueryFunctionBuilder),
        ))
        .add_systems(Startup, setup)
        .run();
}
