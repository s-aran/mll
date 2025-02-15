use crate::Mll;
use mlua::Lua;
use std::collections::HashMap;

#[cfg(feature = "http")]
use chitose::*;

pub fn init(lua: &mut Lua) -> mlua::Result<()> {
    let globals = lua.globals();

    let add_two = lua.create_function(|_, value: i32| Ok(value + 2))?;
    globals.set("add_two", add_two)?;

    #[cfg(feature = "http")]
    {
        let _simple_http_get = lua.create_function(|_, (url, data): (String, String)| {
            let json: serde_json::Value = serde_json::from_str(&data).unwrap();
            let c = "";
            let t = HashMap::<&str, &str>::new();

            Ok(chitose::sync_http_get(&url, &c, t, &data))
        })?;
        globals.set("simple_http_get", _simple_http_get)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::{collections::HashMap, hash::Hash};

    use super::*;

    #[test]
    fn test_add_two_outer() {
        let template = r#"4 + 2 = {{sum}}"#;
        let script = r#"sum = add_two(4)"#;

        let mut mll = Mll::new();
        mll.set_template(template.to_string());
        mll.set_pre_process_script(script.to_string());
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
}
