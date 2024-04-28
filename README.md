Experimental scripting engine for [Bevy](https://github.com/bevyengine/bevy)

```json
{
  "damage": ["+", 2, ["@", "health"]]
}
```

```rust
#[derive(Default, Component, Deref, DerefMut)]
pub struct Health(f64);

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
                .with_bundle::<Damage>("damage")
                .with_dependency::<Health>("health")
                .with_dependency::<Damage>("damage")
                .with_function("+", function::add())
                .with_function("@", function::query()),
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
```
