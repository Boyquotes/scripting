use bevy::prelude::*;
use scripting::{
    LoadScript, Register, ScriptBundle, ScriptComponent, ScriptPlugin, ScriptsReady,
};
use serde::Deserialize;

#[derive(Default, Component)]
struct Invincible;

impl ScriptComponent for Invincible {
    type Data = InvincibleData;
}

#[derive(Deserialize)]
struct InvincibleData;

impl Register for InvincibleData {
    fn register<C: Component>(
        self,
        _registry: &scripting::Registry,
        _asset_server: &AssetServer,
        entity_commands: &mut bevy::ecs::system::EntityCommands,
    ) {
        entity_commands.insert(Invincible);
    }
}

#[derive(Component, Default)]
pub struct OnEquip;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ScriptPlugin::default()
                .with_component::<Invincible>("invincible")
                .with_event::<OnEquip>("on_equip"),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_sword,debug_invincible))
        .run();
}

fn setup(mut asset_events: EventWriter<LoadScript>) {
    asset_events.send(LoadScript::new("sword_of_invincibility.json"));
}

fn spawn_sword(mut commands: Commands, mut events: EventReader<ScriptsReady>) {
    for _event in events.read() {
        commands.spawn((OnEquip, ScriptBundle::new("sword_of_invincibility")));
    }
}


fn debug_invincible(query: Query<Ref<Invincible>, Changed<Invincible>>) {
    for e in &query {
        dbg!(e.is_added());
    }
}
