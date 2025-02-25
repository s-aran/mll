use std::collections::HashMap;

use mlua::Lua;

use crate::utils::json_str_to_lua_table;

use super::builtin::*;

pub struct SimpleHttpGet;

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
