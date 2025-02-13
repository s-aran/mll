use crate::Mll;
use mlua::{Lua, Result};

#[cfg(feature = "chitose")]
use chitose::*;

pub fn init(lua: &mut Lua) -> mlua::Result<()> {
    let globals = lua.globals();

    let add_two = lua.create_function(|_, value: i32| Ok(value + 2))?;
    globals.set("add_two", add_two)?;

    #[cfg(feature = "chitose")]
    {
        let simple_http_get =
            lua.create_function(|_, (url, data): (String, String)| Ok(chitose::sync_http_get()));
        globals.set("simple_http_get", simple_http_get)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::{collections::HashMap, hash::Hash};

    use super::*;

    #[test]
    fn test_values_lua_table() {
        // let template = r#"Hello, {{simple_http_get('https://httpbin.org/get', '{"foo":"bar"}')}}!"#;
        let template = r#"Hello, {{add_two(2)}}!"#;

        let table = HashMap::<&str, String>::new();

        let mut mll = Mll::new(template.to_string());
        let rendered = mll.render(&table);

        println!("{:?}", mll.get_rendered_tags());

        assert_eq!("Hello, 4!", rendered.unwrap());
    }
}
