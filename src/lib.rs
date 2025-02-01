use liquid::model::Value;
use liquid::reflection::ParserReflection;
use liquid_core::runtime::RuntimeBuilder;
use liquid_core::Runtime;
use serde::Deserialize;
use serde_json::from_reader;
use std::fs::File;
use std::io::BufReader;
use std::{collections::HashMap, path::PathBuf};

mod include_tag;

struct LiquidWrapper {
    parser: liquid::Parser,
    obj: liquid::Object,
}

impl Default for LiquidWrapper {
    fn default() -> Self {
        let parser = liquid::ParserBuilder::with_stdlib()
            .tag(include_tag::IncludeTag)
            .build()
            .unwrap();

        let obj = liquid::Object::new();

        Self { parser, obj }
    }
}

impl LiquidWrapper {
    pub fn add_pair(&mut self, key: impl Into<String>, value: Value) -> &mut Self {
        self.obj.insert(key.into().into(), value);

        self
    }

    pub fn get_keys(&self) -> Vec<String> {
        let result = self
            .obj
            .keys()
            .map(|e| e.to_string())
            .collect::<Vec<String>>();
        result
    }
}

struct Lq2Json {
    lq: LiquidWrapper,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_tag() {
        let template = liquid::ParserBuilder::with_stdlib()
            .tag(include_tag::IncludeTag)
            .build()
            .unwrap()
            .parse(r#"{% include "LICENSE" %}"#)
            .unwrap();

        let mut obj = liquid::Object::new();
        let rendered = template.render(&obj).unwrap();

        assert_eq!(
            rendered.to_string(),
            r#"MIT License

Copyright (c) 2024 Sumiishi Aran

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#
        );
    }

    #[test]
    fn test_include_tag_with_parse_file() {
        let template = liquid::ParserBuilder::with_stdlib()
            .tag(include_tag::IncludeTag)
            .build()
            .unwrap()
            .parse_file("template.json")
            .unwrap();

        let mut obj = liquid::Object::new();
        let value = Value::scalar("hoge");
        obj.insert("name".into(), value);
        let rendered = template.render(&obj).unwrap();

        assert_eq!(
            r#"{
  "name": "hoge",
  "data": {
  "foo": "bar"
}

}
"#,
            rendered,
        );
    }

    #[test]
    fn test_keys() {
        let builder = liquid::ParserBuilder::with_stdlib()
            .tag(include_tag::IncludeTag)
            .build()
            .unwrap();

        let template = builder.parse_file("template.json").unwrap();

        // let tags = builder.tags().last().expect("not found").tag();
        let runtime = RuntimeBuilder::new();
        let runtime = runtime.build();

        // println!("{:?}", runtime.roots().get(&0).unwrap().into_string());
        println!("{}", runtime.register

        assert!(false)
    }
}
