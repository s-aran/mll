//! Lua utility commands
//!
//! # Examples
//!
//! ```lua
//! local table = json_to_table('{"key": "value"}')
//! local json = table_to_json(table)
//! ```

use mlua::Lua;
use mlua::Table;

use crate::utils::json_str_to_lua_table;
use crate::utils::lua_table_to_json_str;

use super::builtin::*;

pub struct TableToJson;

impl BuiltinFunction for TableToJson {
    fn get_name(&self) -> &str {
        "table_to_json"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, table: Table| {
                Ok(
                    serde_json::to_string(&lua_table_to_json_str(&lua_ref, table).unwrap())
                        .unwrap(),
                )
            })
            .unwrap()
    }
}

pub struct JsonToTable;

impl BuiltinFunction for JsonToTable {
    fn get_name(&self) -> &str {
        "json_to_table"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, json: String| Ok(json_str_to_lua_table(&lua_ref, &json)))
            .unwrap()
    }
}
