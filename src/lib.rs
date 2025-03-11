mod builtin;
mod builtins;
mod utils;

use mlua::{FromLua, Lua, Table};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::path::Path;
use uuid::Uuid;

pub trait GetValueByName<T> {
    fn get_by_name(&self, name: &str) -> Option<T>;
}

impl<T> GetValueByName<T> for HashMap<&str, T>
where
    T: Clone,
{
    fn get_by_name(&self, name: &str) -> Option<T> {
        match self.get(name) {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }
}

impl<T> GetValueByName<T> for Table
where
    T: FromLua,
{
    fn get_by_name(&self, name: &str) -> Option<T> {
        match self.get::<T>(name) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }
}

struct Internal {
    lua: Lua,
}

impl Internal {
    pub fn new() -> Self {
        let lua = Lua::new();
        let _ = builtin::Builtins::init(&lua);

        Self { lua }
    }

    pub fn load_script(&self, script: &str) -> Result<(), String> {
        let result = self.lua.load(script).exec();
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub struct Mll {
    template: String,
    pre_process_script: String,
    tags: HashMap<String, String>,
    processed_tags: HashSet<String>,
}

impl Mll {
    pub fn new() -> Self {
        Self {
            template: String::new(),
            pre_process_script: String::new(),
            tags: HashMap::new(),
            processed_tags: HashSet::new(),
        }
    }

    pub fn pre_process_script(&self) -> &String {
        &self.pre_process_script
    }

    pub fn set_pre_process_script(&mut self, script: String) {
        self.pre_process_script = script;
    }

    pub fn template(&self) -> &String {
        &self.template
    }

    pub fn set_template(&mut self, template: String) {
        self.template = template;
    }

    pub fn read_template_file(&mut self, path: &Path) -> &Self {
        let template = read_to_string(path).unwrap();
        self.set_template(template);

        self
    }

    pub fn render_with_lua(&mut self, script: &str) -> Result<String, String> {
        let internal = Internal::new();

        let result = internal.load_script(script);
        match result {
            Ok(_) => {
                let _ = internal.load_script(script);
                let table = internal.lua.globals();
                let rendered = self.render(&table);
                rendered
            }
            Err(e) => Err(e),
        }
    }

    pub fn render_lua_globals(&mut self) -> Result<String, String> {
        let internal = Internal::new();
        let _ = internal.load_script(&self.pre_process_script);

        let table = internal.lua.globals();

        // print
        // for pair in table.pairs::<String, mlua::Value>() {
        //     let (key, value) = pair.unwrap();
        //     println!("{}: {:?}", key, value);
        // }

        self.render(&table)
    }

    pub fn render<T>(&mut self, table: &T) -> Result<String, String>
    where
        T: GetValueByName<String>,
    {
        // define regex pattern for Mustache's variable-like syntax (e.g. {{ name }})
        let re_variable = Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();
        // let re_function = Regex::new(r#"\{\{\s*([\w\(\)"]+)\s*\}\}"#).unwrap();

        let internal = Internal::new();

        // internal.load_script(&self.pre_process_script);

        let mut succeeded = true;
        let rendered = re_variable
            .replace_all(&self.template.as_str(), |caps: &regex::Captures| {
                // extract variable name (or Lua script) from template
                let tag = caps.get(1).unwrap().as_str();

                // make temporary variable name
                let uuid = Uuid::new_v4();
                let variable_name = format!(
                    "v_{}",
                    uuid.simple().encode_lower(&mut Uuid::encode_buffer())
                );

                // map variable name and temporary variable
                self.tags.insert(variable_name.to_string(), tag.to_string());

                // set Lua global table
                // TODO: replace following script, too general purpose:
                //  local function f_uuid()
                //      return tag
                //  end
                //  v_uuid = f_uuid()
                let result = internal.lua.load(format!("{variable_name} = {tag}")).exec();
                match result {
                    Ok(_) => match table.get_by_name(tag) {
                        Some(value) => {
                            self.processed_tags.insert(tag.to_owned());
                            value
                        }
                        None => {
                            succeeded = false;
                            eprintln!("variable not found: {}", tag);
                            return "".to_string();
                        }
                    },
                    Err(e) => {
                        succeeded = false;
                        eprintln!("result: {}", e);
                        return "".to_string();
                    }
                }
            })
            .into_owned();

        // let rendered = re_function
        //     .replace_all(rendered.as_str(), |caps: &regex::Captures| {
        //         let calling = caps.get(1).unwrap().as_str();

        //         // make temporary variable name
        //         let uuid = Uuid::new_v4();
        //         let variable_name = format!(
        //             "f_{}",
        //             uuid.simple().encode_lower(&mut Uuid::encode_buffer())
        //         );

        //         // map variable name and temporary variable
        //         self.tags
        //             .insert(variable_name.to_string(), calling.to_string());

        //         // set Lua global table
        //         let result = internal
        //             .lua
        //             .load(format!("{variable_name} = {calling}"))
        //             .exec();
        //         match result {
        //             Ok(_) => match internal.lua.globals().get(variable_name) {
        //                 Ok(value) => value,
        //                 Err(e) => {
        //                     eprintln!("{}", e);
        //                     "".to_string()
        //                 }
        //             },
        //             Err(e) => {
        //                 eprintln!("result: {}", e);
        //                 return "".to_string();
        //             }
        //         }
        //     })
        //     .into_owned();

        if succeeded {
            Ok(rendered)
        } else {
            Err(rendered)
        }
    }

    pub fn get_rendered_tags(&self) -> Vec<String> {
        self.tags.values().cloned().collect()
    }

    pub fn get_missing_variables(&self) -> Vec<String> {
        let mut missing_variables = Vec::new();

        for tag in self.tags.values() {
            if !self.processed_tags.contains(tag) {
                missing_variables.push(tag.to_string());
            }
        }

        missing_variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_values_lua_table() {
        let template = "Hello, {{name}}!";

        let lua = Lua::new();
        lua.load("name = 'hoge'").exec().unwrap();

        let mut mll = Mll::new();
        mll.set_template(template.to_string());
        let rendered = mll.render(&lua.globals());
        assert_eq!("Hello, hoge!", rendered.unwrap());

        let tags = mll.get_rendered_tags();
        assert!(tags.contains(&"name".to_string()));
        assert_eq!(1, tags.len());
    }

    #[test]
    fn test_values_hashmap() {
        let template = "Hello, {{name}}!";

        let mut table = HashMap::new();
        table.insert("name", "hoge".to_string());

        let mut mll = Mll::new();
        mll.set_template(template.to_string());
        let rendered = mll.render(&table);
        assert_eq!("Hello, hoge!", rendered.unwrap());

        let tags = mll.get_rendered_tags();
        assert!(tags.contains(&"name".to_string()));
        assert_eq!(1, tags.len());
    }

    #[test]
    fn test_get_missing_variables() {
        let template = "{{hello}}, {{name}}!";

        let mut table = HashMap::new();
        table.insert("name", "hoge".to_string());

        let mut mll = Mll::new();
        mll.set_template(template.to_string());
        let rendered = mll.render(&table);
        assert!(rendered.is_err());

        let missing_variables = mll.get_missing_variables();
        assert_eq!(1, missing_variables.len());
        assert_eq!("hello", missing_variables[0]);
    }
}
