use mlua::{FromLua, Lua, Table};
use regex::Regex;
use std::collections::HashMap;
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

pub struct Mll {
    template: String,
    tags: HashMap<String, String>,
}

impl Mll {
    pub fn new(template: String) -> Self {
        Self {
            template,
            tags: HashMap::new(),
        }
    }

    pub fn set_template(&mut self, template: String) -> &Self {
        self.template = template;

        self
    }

    pub fn read_template_file(&mut self, path: &Path) -> &Self {
        let template = read_to_string(path).unwrap();
        self.set_template(template);

        self
    }

    pub fn render<T>(&mut self, table: &T) -> Result<String, String>
    where
        T: GetValueByName<String>,
    {
        // define regex pattern for Mustache's variable-like syntax (e.g. {{ name }})
        let re = Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();

        let lua = Lua::new();

        let rendered = re
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
                let result = lua.load(format!("{variable_name} = {tag}")).exec();
                match result {
                    Ok(_) => match table.get_by_name(tag) {
                        Some(value) => value,
                        None => {
                            eprintln!("variable not found: {}", tag);
                            return "".to_string();
                        }
                    },
                    Err(e) => {
                        eprintln!("result: {}", e);
                        return "".to_string();
                    }
                }
            })
            .into_owned();

        Ok(rendered)
    }

    pub fn get_rendered_tags(&self) -> Vec<String> {
        self.tags.values().cloned().collect()
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

        let mut mll = Mll::new(template.to_string());
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

        let mut mll = Mll::new(template.to_string());
        let rendered = mll.render(&table);
        assert_eq!("Hello, hoge!", rendered.unwrap());

        let tags = mll.get_rendered_tags();
        assert!(tags.contains(&"name".to_string()));
        assert_eq!(1, tags.len());
    }
}
