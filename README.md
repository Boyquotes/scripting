Experimental scripting engine for [Bevy](https://github.com/bevyengine/bevy)

```json
{
  "id": "sword",
  "damage": ["-", 1, ["/", ["@", "durability"], ["@", "max_durability"]]]
}
```

```rust
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

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ScriptPlugin::default()
                .with_derived::<Damage>("damage")
                .with_derived::<Durability>("durability")
                .with_derived::<MaxDurability>("max_durability"),
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
            Durability(0.1),
            MaxDurability(1.),
            ScriptBundle::new("sword"),
        ));
    }
}

fn debug(query: Query<&Damage, Changed<Damage>>) {
    for dmg in &query {
        dbg!(dmg.0);
    }
}
```
