# config-schema-gen

Reads a JSON schema and generates a Rust trait with getters of correct types. Convenient for reading configs.

For example, following JSON schema:

```json
{
  "type": "object",
  "properties": {
    "my-string": {
      "type": "string"
    },
    "my-string-with-default": {
      "type": "string",
      "default": "blah"
    },
    "my-int": {
      "type": "integer",
      "default": 15
    },
    "my-bool": {
      "type": "boolean",
      "default": false
    }
  }
}
```

would generate procedural macro equivalent to:

```rust
trait TypedConfig {
  fn my_string(&self) -> Option<String>;

  fn my_string_with_default(&self) -> String;

  fn my_int(&self) -> i64;

  fn my_bool(&self) -> bool;
}
```

## Usage

Just import the procedural macro with path to your config schema file.

```rust
embed_typed_config!("config.schema.json");
```

Generated `TypedConfig` trait has unimplemented functions `get_bool`, `get_int` and `get_str`.

If you are using crate [config](https://github.com/mehcode/config-rs) you should write:

```rust
impl TypedConfig for Config {
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get::<bool>(key).ok()
    }

    fn get_int(&self, key: &str) -> Option<i64> {
        self.get::<i64>(key).ok()
    }

    fn get_str(&self, key: &str) -> Option<String> {
        self.get::<String>(key).ok()
    }
}
```

To get a config field, you can now use the generated functions:

```rust
let conf: Config = ...;

let my_bool: bool = conf.my_bool();
let my_string: Option<String> = conf.my_string();
```

All dashes and dots (including sub-object separators) are replaced with underscores. This means that schema:

```json
{
  "type": "object",
  "properties": {
    "my-sub.object": {
      "type": "object",
      "properties": {
        "my-string": {
          "type": "string"
        }
      }
    }
  }
}
```

would produce a function with name `my_sub_object_my_string`.