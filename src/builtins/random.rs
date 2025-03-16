//! Generate random numbers and strings commands
//!
//! # Examples
//!
//! ```lua
//! local random_int = random_int(1, 100)
//! print(random_int)   -- e.g. 42
//!
//! local random_string = random_string(10)
//! print(random_string)   -- e.g. "aBcDeFgHiJ"
//! ```

use mlua::Lua;
use rand::prelude::*;

use super::builtin::*;

pub struct RandomInt;

impl BuiltinFunction for RandomInt {
    fn get_name(&self) -> &str {
        "random_int"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, (minimum, maximum): (i32, i32)| {
                let mut rng = rand::rng();
                let rand = rng.random_range(minimum..maximum);

                Ok(rand)
            })
            .unwrap()
    }
}

pub struct RandomString;

impl BuiltinFunction for RandomString {
    fn get_name(&self) -> &str {
        "random_string"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, length: usize| {
                let rng = rand::rng();
                let str = rng.sample_iter(rand::distr::Alphanumeric).take(length);
                Ok(String::from_utf8(str.collect()).unwrap())
            })
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::Mll;

    #[test]
    fn test_random_int() {
        let mut mll = Mll::new();

        let template = r#"{{a}},{{b}},{{c}}"#;
        let script = r#"
            a = random_int(1, 100)
            b = random_int(-100, 0)
            c = random_int(-100, 100)
        "#;

        mll.set_template(template.to_string());
        let rendered = mll.render_with_lua(script);

        let split = rendered
            .unwrap()
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        assert_eq!(3, split.len());

        assert!(split[0].parse::<i32>().unwrap() >= 1);
        assert!(split[0].parse::<i32>().unwrap() <= 100);

        assert!(split[1].parse::<i32>().unwrap() >= -100);
        assert!(split[1].parse::<i32>().unwrap() <= 0);

        assert!(split[2].parse::<i32>().unwrap() >= -100);
        assert!(split[2].parse::<i32>().unwrap() <= 100);

        assert_ne!(split[0], split[1]);
        assert_ne!(split[0], split[2]);
        assert_ne!(split[1], split[2]);
    }

    #[test]
    fn test_random_string() {
        let mut mll = Mll::new();

        let template = r#"{{a}},{{b}},{{c}}"#;
        let script = r#"
            a = random_string(10)
            b = random_string(20)
            c = random_string(30)
        "#;

        mll.set_template(template.to_string());
        let rendered = mll.render_with_lua(script);

        let split = rendered
            .unwrap()
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        assert_eq!(3, split.len());

        assert_eq!(10, split[0].len());
        assert_eq!(20, split[1].len());
        assert_eq!(30, split[2].len());

        assert!(!split[1].contains(&split[0]));
        assert!(!split[2].contains(&split[0]));
    }
}
