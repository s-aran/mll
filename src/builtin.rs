use crate::builtins::{add_two::AddTwo, builtin::BuiltinFunction, exec::Exec};
use mlua::Lua;

pub fn init(lua: &Lua) -> mlua::Result<()> {
    let _ = AddTwo {}.set_function(lua);

    let _ = Exec {}.set_function(lua);

    #[cfg(feature = "http")]
    {
        use crate::builtins::simple_http::SimpleHttpGet;
        let _ = SimpleHttpGet {}.set_function(lua);
    }

    #[cfg(feature = "datetime")]
    {
        use crate::builtins::datetime::DateTimeFormat;
        let _ = DateTimeFormat {}.set_function(lua);

        use crate::builtins::datetime::DateTimeOffset;
        let _ = DateTimeOffset {}.set_function(lua);
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{Mll, utils::json_to_lua};

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
