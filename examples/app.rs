use bevy::prelude::*;
use scripting::{
    expr::{
        function::{AddFunctionBuilder, QueryFunctionBuilder},
        ExprData, StaticExpr,
    },
    Registry, Scope, ScriptPlugin,
};

#[derive(PartialEq, Component)]
pub struct Health(f64);

#[derive(Resource)]
struct SwordHandle(Handle<ExprData>);

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
        .add_systems(Update, (spawn_expr, run_expr, debug_health))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("sword.json");
    commands.insert_resource(SwordHandle(handle));
}

fn spawn_expr(
    mut commands: Commands,
    mut expr_data_assets: ResMut<Assets<ExprData>>,
    registry: Res<Registry>,
) {
    let mut asset_ids = Vec::new();

    for (asset_id, expr_data) in expr_data_assets.iter_mut() {
        let mut entity_commands = commands.spawn(Health(1.));
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

fn run_expr(mut query: Query<(&mut Health, &Scope), Changed<Scope>>) {
    for (mut health, expr) in &mut query {
        if let Some(StaticExpr::Number(new_health)) = expr.run() {
            health.set_if_neq(Health(new_health));
        }
    }
}

fn debug_health(query: Query<&Health, Changed<Health>>) {
    for health in &query {
        dbg!(health.0);
    }
}
