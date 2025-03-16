//! Render a template with parameters command
//!
//! # Example
//! ```lua
//! content = render("{{name}}", {name="John Doe"})
//! print(content)  -- John Doe
//! ```

use core::panic;
use std::{fs, path::PathBuf};

use mlua::{Lua, Table, Value};

use crate::Mll;

use super::builtin::*;

pub struct Render;

impl BuiltinFunction for Render {
    fn get_name(&self) -> &str {
        "render"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(
                move |_, (template, params_path_or_table): (String, Value)| {
                    match params_path_or_table {
                        Value::String(p) => {
                            let path = PathBuf::from(p.to_string_lossy());
                            return Ok(render_with_file(template, path));
                        }
                        Value::Table(t) => {
                            return Ok(render_with_table(template, t));
                        }
                        _ => {
                            panic!("Unexpected data type");
                        }
                    }
                },
            )
            .unwrap()
    }
}

fn render_with_file(template_content: String, params_path: PathBuf) -> Result<String, String> {
    let params_content = match fs::read_to_string(params_path) {
        Ok(c) => c,
        Err(e) => panic!("Error reading file: {}", e),
    };

    let mut mll = Mll::new();
    mll.set_template(template_content);
    let result = mll.render_with_lua(&params_content);

    result
}

fn render_with_table(template_content: String, params: Table) -> Result<String, String> {
    let mut mll = Mll::new();
    mll.set_template(template_content);
    let result = mll.render(&params);

    result
}

#[cfg(test)]
mod tests {
    use crate::Mll;

    #[test]
    fn test_render_with_table() {
        let template = r#"{"data": {{content}}, "name": "{{name}}", "age": {{age}}}"#;
        let script = r#"
            local table = {
                name="John Doe",
                age=21,
            }
            name = "Jane Doe"
            age = 20
            content = render('{"name": "{{name}}", "age": {{age}}}', table)
        "#;

        let mut mll = Mll::new();
        mll.set_template(template.to_string());

        let render = mll.render_with_lua(script);

        let expected =
            r#"{"data": {"name": "John Doe", "age": 21}, "name": "Jane Doe", "age": 20}"#;

        assert_eq!(expected, render.unwrap());
    }

    #[test]
    #[should_panic(expected = "Unexpected data type")]
    fn test_render_panic() {
        let template = r#"{{content}}"#;
        let script = r#"
            content = render("", nil)
        "#;

        let mut mll = Mll::new();
        mll.set_template(template.to_string());

        let _ = mll.render_with_lua(script);
    }
}
