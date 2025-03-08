use mlua::Lua;

use super::builtin::BuiltinFunction;

pub struct Sql;

impl BuiltinFunction for AddTwo {
    fn get_name(&self) -> &str {
        "execute_sql"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, query: String| Ok(value + 2))
            .unwrap()
    }
}

trait ConnectionStringBuilder {
    fn build(&self) -> String;
}

struct MySql {
    host: String,
    port: u32,
    database: String,
    username: String,
    password: String,
}

impl ConnectionStringBuilder for MySql {
    fn build(&self) -> String {
        format!("mysql://{self.username}:{self.password}@{self.host}:{self.port}/{self.database}")
    }
}

fn execute_sql() {
    //
}
