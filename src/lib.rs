use mlua::prelude::*;
use mustache::*;
use serde::Deserialize;
use serde_json::from_reader;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::Mutex;
use std::{collections::HashMap, path::PathBuf};

pub struct Lq {
    template: String,
    table: HashMap<String, String>,
}

impl Lq {
    pub fn new(template: String) -> Self {
        Self {
            template,
            table: HashMap::new(),
        }
    }

    pub fn render(&mut self) -> String {
        let lua = Lua::new();

        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use serde::de::value;

    use super::*;

    #[test]
    fn test_include_tag() {}
}
