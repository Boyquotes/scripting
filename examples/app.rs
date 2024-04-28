use bevy::prelude::*;
use scripting::{
    expr::ExprData, DynamicComponent, LoadScript, ScriptBundle, ScriptPlugin, ScriptsReady,
};

#[derive(Default, Component, Deref, DerefMut)]
pub struct Health(f64);

impl DynamicComponent for Health {
    type Data = ExprData;
}

#[derive(Default, Component, Deref, DerefMut)]
pub struct Damage(f64);

impl DynamicComponent for Damage {
    type Data = ExprData;
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ScriptPlugin::default()
                .with_component::<Damage>("damage")
                .with_component::<Health>("health"),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_sword, debug))
        .run();
}

fn setup(mut asset_events: EventWriter<LoadScript>) {
    asset_events.send(LoadScript::new("sword.json"));
}

fn spawn_sword(mut commands: Commands, mut events: EventReader<ScriptsReady>) {
    for _event in events.read() {
        commands.spawn((
            Health(10.),
            Damage(1.),
            ScriptBundle(String::from("sword.json")),
        ));
    }
}

fn debug(query: Query<&Damage, Changed<Damage>>) {
    for dmg in &query {
        dbg!(dmg.0);
    }
}
