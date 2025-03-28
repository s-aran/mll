use mlua::{Lua, Result, Table, Value};
use serde_json::{Map, Value as JsonValue};

use encoding_rs;
use encoding_rs::SHIFT_JIS;

/// Convert a JSON string to a Lua table
///
/// # Arguments
///
/// * `lua` - A reference to the Lua instance
/// * `json_str` - A JSON string
///
/// # Returns
///
/// `Result<Table>` - The Lua table
pub fn json_str_to_lua_table(lua: &Lua, json_str: &str) -> Result<Table> {
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

/// Lua table to JSON string
///
/// # Arguments
///
/// * `lua` - A reference to the Lua instance
/// * `table` - A Lua table
///
/// # Returns
///
/// `Result<String>` - The JSON string
pub fn lua_table_to_json_str(_: &Lua, table: Table) -> Result<String> {
    let json_value = lua_to_json(Value::Table(table))?;
    serde_json::to_string(&json_value).map_err(|e| mlua::Error::RuntimeError(e.to_string()))
}

pub fn json_to_lua(lua: &Lua, json: &JsonValue) -> Result<Value> {
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

pub fn lua_to_json(value: Value) -> Result<JsonValue> {
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

/// Convert a Lua string to a Shift-JIS string
///
/// # Arguments
///
/// * `lua` - A reference to the Lua instance
/// * `string` - A Lua string
///
/// # Returns
///
/// `String` - The Shift-JIS string
pub fn lua_string_to_shift_jis(lua: &Lua, string: mlua::String) -> mlua::String {
    let ls = string.to_str().unwrap();
    let (s, _, _) = SHIFT_JIS.encode(&ls);
    lua.create_string(&s).unwrap()
}

pub fn do_blocking<F>(future: F) -> F::Output
where
    F: Future,
{
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(future)
}
