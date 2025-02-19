use mlua::Lua;

use super::builtin::BuiltinFunction;

pub struct AddTwo;

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
