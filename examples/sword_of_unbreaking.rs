use bevy::prelude::*;
use scripting::{
    expr::ExprData, LoadScript, ScriptBundle, ScriptComponent, ScriptPlugin, ScriptsReady,
};


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
                .with_event::<OnEquip>("on_equip"),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_sword, debug))
        .run();
}

fn setup(mut asset_events: EventWriter<LoadScript>) {
    asset_events.send(LoadScript::new("sword_of_unbreaking.json"));
}

fn spawn_sword(mut commands: Commands, mut events: EventReader<ScriptsReady>) {
    for _event in events.read() {
        commands.spawn((
            Durability(0.1),
            MaxDurability(1.),
            OnEquip,
            ScriptBundle::new("sword_of_unbreaking"),
        ));
    }
}

fn debug(query: Query<&Damage, Changed<Damage>>) {
    for dmg in &query {
        dbg!(dmg.0);
    }
}
