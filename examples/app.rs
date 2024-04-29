use bevy::prelude::*;
use scripting::{
    expr::ExprData, LoadScript, Register, ScriptBundle, ScriptComponent, ScriptPlugin, ScriptsReady,
};
use serde::Deserialize;

#[derive(Default, Component, Deref, DerefMut)]
pub struct Durability(f64);

impl ScriptComponent for Durability {
    type Data = ExprData;
}

#[derive(Default, Component, Deref, DerefMut)]
pub struct MaxDurability(f64);

impl ScriptComponent for MaxDurability {
    type Data = ExprData;
}

#[derive(Default, Component, Deref, DerefMut)]
pub struct Damage(f64);

impl ScriptComponent for Damage {
    type Data = ExprData;
}

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
                .with_derived::<Damage>("damage")
                .with_derived::<Durability>("durability")
                .with_derived::<MaxDurability>("max_durability")
                .with_component::<Invincible>("invincible")
                .with_event::<OnEquip>("on_equip"),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_sword, debug, debug_invincible))
        .run();
}

fn setup(mut asset_events: EventWriter<LoadScript>) {
    asset_events.send(LoadScript::new("sword.json"));
}

fn spawn_sword(mut commands: Commands, mut events: EventReader<ScriptsReady>) {
    for _event in events.read() {
        commands.spawn((OnEquip, ScriptBundle::new("sword_of_invincibility")));
    }
}

fn debug(query: Query<&Damage, Changed<Damage>>) {
    for dmg in &query {
        dbg!(dmg.0);
    }
}

fn debug_invincible(query: Query<Ref<Invincible>, Changed<Invincible>>) {
    for e in &query {
        dbg!(e.is_added());
    }
}
