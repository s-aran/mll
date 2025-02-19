use crate::Mll;
use mlua::{Lua, Result, Table, Value};
use serde_json::{Map, Value as JsonValue};
use std::collections::HashMap;

trait BuiltinFunction {
    fn get_name(&self) -> &str;
    fn get_function(&self, lua: &Lua) -> mlua::Function;

    fn set_function(&self, lua: &Lua) -> mlua::Result<()> {
        let globals = lua.globals();
        let name = self.get_name();
        let func = self.get_function(lua);
        globals.set(name, func)?;
        Ok(())
    }
}

struct AddTwo;

impl BuiltinFunction for AddTwo {
    fn get_name(&self) -> &str {
        "add_two"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, value: i32| Ok(value + 2))
            .unwrap()
    }
}

struct SimpleHttpGet;

impl BuiltinFunction for SimpleHttpGet {
    fn get_name(&self) -> &str {
        "simple_http_get"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, (url, data): (String, String)| {
                let c = "";
                let t = HashMap::<&str, &str>::new();

                let r = chitose::sync_http_get(&url, &c, t, &data);
                let r2 = json_str_to_lua_table(&lua_ref, &r);

                Ok(r2.unwrap())
            })
            .unwrap()
    }
}

pub fn init(lua: &Lua) -> mlua::Result<()> {
    let _ = AddTwo {}.set_function(lua);

    #[cfg(feature = "http")]
    let _ = SimpleHttpGet {}.set_function(lua);

    Ok(())
}

fn json_str_to_lua_table(lua: &Lua, json_str: &str) -> Result<Table> {
    let json_value: JsonValue = serde_json::from_str(json_str)
        .map_err(|e| mlua::Error::RuntimeError(format!("JSON parse error: {}", e)))?;

    let lua_value = json_to_lua(lua, &json_value)?;
    match lua_value {
        Value::Table(table) => Ok(table),
        _ => Err(mlua::Error::RuntimeError(
            "Expected JSON to be a table".into(),
        )),
    }
}

fn json_to_lua(lua: &Lua, json: &JsonValue) -> Result<Value> {
    match json {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Err(mlua::Error::RuntimeError("Invalid number".into()))
            }
        }
        JsonValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
        JsonValue::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj.iter() {
                table.set(k.as_str(), json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

fn lua_to_json(value: Value) -> Result<JsonValue> {
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(b)),
        Value::Integer(i) => Ok(JsonValue::Number(i.into())),
        Value::Number(n) => Ok(JsonValue::Number(
            serde_json::Number::from_f64(n)
                .ok_or_else(|| mlua::Error::RuntimeError("Invalid number".into()))?,
        )),
        Value::String(s) => Ok(JsonValue::String(s.to_str()?.to_string())),
        Value::Table(table) => {
            if is_array(&table)? {
                let mut arr = Vec::new();
                for pair in table.pairs::<i64, Value>() {
                    let (_, v) = pair?;
                    arr.push(lua_to_json(v)?);
                }
                Ok(JsonValue::Array(arr))
            } else {
                let mut obj = Map::new();
                for pair in table.pairs::<String, Value>() {
                    let (k, v) = pair?;
                    obj.insert(k, lua_to_json(v)?);
                }
                Ok(JsonValue::Object(obj))
            }
        }
        _ => Err(mlua::Error::RuntimeError("Unsupported Lua value".into())),
    }
}

fn is_array(table: &Table) -> Result<bool> {
    let mut expected = 1;
    for pair in table.pairs::<Value, Value>() {
        let (k, _) = pair?;
        if let Value::Integer(i) = k {
            if i != expected {
                return Ok(false);
            }
            expected += 1;
        } else {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_add_two_outer() {
        let template = r#"4 + 2 = {{sum}}"#;
        let script = r#"sum = add_two(4)"#;

        let mut mll = Mll::new();
        mll.set_template(template.to_string());
        mll.set_pre_process_script(script.to_string());

        assert_eq!("4 + 2 = 6", mll.render_lua_globals().unwrap());
    }

    #[test]
    fn test_simple_http_get() {
        let template = "{{value}}";
        let pre_process_script = r#"
            response = simple_http_get("https://httpbin.org/get", '{"foo": "bar"}')
            args = response['args']
            value = args['foo']
        "#;

        let mut mll = Mll::new();
        mll.set_template(template.to_string());
        mll.set_pre_process_script(pre_process_script.to_string());

        // mll.get_variable("response").unwrap();

        assert_eq!("bar", mll.render_lua_globals().unwrap());
    }

    // #[test]
    // fn test_add_two() {
    //     let template = r#"4 + 2 = {{add_two(4)}}"#;
    //     let table = HashMap::<&str, String>::new();

    //     let mut mll = Mll::new(template.to_string());
    //     let rendered = mll.render(&table);

    //     assert_eq!("4 + 2 = 6", rendered.unwrap());
    // }

    // #[test]
    // fn test_simple_http_get() {
    //     let template = r#"Get ==> {{simple_http_get('https://httpbin.org/get', '{"foo":"bar"}')}}"#;
    //     let table = HashMap::<&str, String>::new();

    //     let mut mll = Mll::new(template.to_string());
    //     let rendered = mll.render(&table);

    //     assert_eq!("4 + 2 = 6", rendered.unwrap());
    // }

    #[test]
    fn test_serde_value_pass_lua() {
        let json_str = r#"{
            "name": "hoge",
            "age": 20,
            "is_male": true,
            "tags": ["hoge", "fuga", "piyo"]
        }"#;
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        println!("{:?}", json);

        let lua = Lua::new();
        let globals = lua.globals();

        let v = json_to_lua(&lua, &json).unwrap();
        println!("{:?}", v);

        //     assert!(false);
    }
}
