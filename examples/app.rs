use bevy::prelude::*;
use scripting::{
    expr::{function, ExprData},
    AssetRegistry, DynamicComponent, Registry, ScriptBundle, ScriptPlugin,
};

#[derive(Default, Component, Deref, DerefMut)]
pub struct Health(f64);

#[derive(Default, Component, Deref, DerefMut)]
pub struct Damage(f64);

impl DynamicComponent for Damage {
    type Data = ExprData;

    fn register(
        data: Self::Data,
        registry: &Registry,
        entity_commands: &mut bevy::ecs::system::EntityCommands,
    ) {
        data.build(registry)
            .spawn::<Self>(registry, entity_commands)
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ScriptPlugin::default()
                .with_bundle::<Damage>("damage")
                .with_dependency::<Health>("health")
                .with_dependency::<Damage>("damage")
                .with_function("+", function::add())
                .with_function("@", function::query()),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, debug)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut asset_registry: ResMut<AssetRegistry>,
) {
    commands.spawn((
        Health(10.),
        Damage(1.),
        ScriptBundle(String::from("sword.json")),
    ));

    let handle = asset_server.load("sword.json");
    asset_registry
        .handles
        .insert(String::from("sword.json"), handle);
}

fn debug(query: Query<&Damage, Changed<Damage>>) {
    for dmg in &query {
        dbg!(dmg.0);
    }
}
