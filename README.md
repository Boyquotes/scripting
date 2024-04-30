Experimental scripting engine for [Bevy](https://github.com/bevyengine/bevy)

```json
{
  "id": "sword_of_invincibility",
  "durability": 0.1,
  "max_durability": 1,
  "damage": ["-", 1, ["/", ["@", "durability"], ["@", "max_durability"]]],
  "on_equip": {
    "add": "invincible"
  }
}
```

# Introduction
Components are defined as JSON maps and can be deserialized into ECS components with [Serde](https://serde.rs/);

```json
{
  "id": "sword",
  "durability": 0.5,
  "max_durability": 2,
  "damage": 1.5
}
```

Expressions are reactive and use a LISP-like syntax.
 * Functions are written with `["{NAME}", "{ARG 1}", "{ARG 2}", ...]
 * Queries are performed with the function `@`

```json
{
  "id": "sword_of_invincibility",
  "damage": ["-", 1, ["/", ["@", "durability"], ["@", "max_durability"]]]
}
```

Components can also trigger events that can interact with the ECS.

```json
{
  "id": "sword_of_invincibility",
  "on_equip": {
    "add": "invincible"
  }
}
```
