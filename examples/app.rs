use bevy::prelude::*;
use scripting::{
    expr::{function, ExprData},
    Registry, ScriptPlugin,
};

#[derive(Default, Component, Deref, DerefMut)]
pub struct Health(f64);

#[derive(Default, Component, Deref, DerefMut)]
pub struct Damage(f64);

#[derive(Resource)]
struct SwordHandle(Handle<ExprData>);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ScriptPlugin::default()
                .with_dependency::<Health>("health")
                .with_dependency::<Damage>("damage")
                .with_function("+", function::add())
                .with_function("@", function::query()),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_expr, debug))
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
        let mut entity_commands = commands.spawn((Health(10.), Damage(1.)));
        expr_data
            .clone()
            .build(&registry)
            .spawn::<Damage>(&registry, &mut entity_commands);

        asset_ids.push(asset_id);
    }

    for id in asset_ids {
        expr_data_assets.remove(id);
    }
}

fn debug(query: Query<&Damage, Changed<Damage>>) {
    for dmg in &query {
        dbg!(dmg.0);
    }
}
