use clap::{arg, command, Parser};
use liquid::model::Value;
use serde::Deserialize;
use serde_json::from_reader;
use std::fs::File;
use std::io::BufReader;
use std::{collections::HashMap, path::PathBuf};

fn main() {
    let args = Args::parse();

    if !args.is_template_passed() {
        eprintln!("template file is not passed");
        return;
    }

    let template = match liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse_file(args.get_template_file_path())
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("failed to parse template file: {}", e);
            return;
        }
    };

    let mut obj = liquid::Object::new();
    for (k, v) in args.get_parameters().map {
        obj.insert(
            k.into(),
            match v {
                serde_json::Value::Number(n) => Value::scalar(n.as_i64().unwrap()),
                serde_json::Value::String(s) => Value::scalar(s),
                serde_json::Value::Bool(b) => Value::Scalar(b.into()),
                _ => Value::Nil,
            },
        );
    }

    let rendered = template.render(&obj).unwrap();

    println!("{}", rendered.to_string());
}

#[derive(Clone, Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long, help = "template file (*.liquid)")]
    template: Option<String>,

    #[arg(short, long, help = "parameter file (*.json)")]
    parameter: Option<String>,

    #[arg(short, long, help = "output file (*.json)")]
    output: Option<String>,

    #[arg(help = "parameters (key=value ...)")]
    params: Vec<String>,
}

impl Args {
    pub fn is_template_passed(&self) -> bool {
        self.template.is_some() && self.template.as_ref().unwrap().len() > 0
    }

    pub fn is_parameter_passed(&self) -> bool {
        self.parameter.is_some() && self.parameter.as_ref().unwrap().len() > 0
    }

    pub fn is_output_passed(&self) -> bool {
        self.output.is_some() && self.output.as_ref().unwrap().len() > 0
    }

    pub fn is_parameters_passed(&self) -> bool {
        self.params.len() > 0
    }

    pub fn get_template_file_path(&self) -> PathBuf {
        PathBuf::from(self.template.clone().unwrap())
    }

    pub fn get_parameter_file_path(&self) -> PathBuf {
        PathBuf::from(self.parameter.clone().unwrap())
    }

    pub fn get_output_file_path(&self) -> PathBuf {
        PathBuf::from(self.output.clone().unwrap())
    }

    fn try_parse_parameter_file(&self) -> Result<Parameters, String> {
        let file = match File::open(self.get_parameter_file_path()) {
            Ok(f) => f,
            Err(e) => {
                return Err(format!("{}", e));
            }
        };

        let reader: BufReader<File> = BufReader::new(file);

        match from_reader::<BufReader<File>, Parameters>(reader) {
            Ok(p) => Ok(p),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn split_key_value(pair: impl Into<String>) -> (String, String) {
        let split = pair
            .into()
            .split('=')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        if split.len() > 2 {
            let key = split[0].clone();
            let value = split[1..].join("=");

            return (key, value);
        }

        return (split[0].clone(), split[1].clone());
    }

    fn make_parameters_from_params(&self) -> Parameters {
        let mut map = HashMap::new();

        for p in self.params.iter() {
            let (k, v) = Args::split_key_value(p);
            map.insert(k, serde_json::Value::String(v));
        }

        Parameters::new_with_map(map)
    }

    pub fn get_parameters(&self) -> Parameters {
        if self.is_parameter_passed() && self.is_parameters_passed() {
            let mut params = self.try_parse_parameter_file().unwrap();
            params.map.extend(self.make_parameters_from_params().map);
            return params;
        }

        if self.is_parameters_passed() {
            return self.make_parameters_from_params();
        }

        if self.is_parameter_passed() {
            return self.try_parse_parameter_file().unwrap();
        }

        Parameters::new()
    }
}

#[derive(Deserialize, Debug)]
struct Parameters {
    #[serde(flatten)]
    map: HashMap<String, serde_json::Value>,
}

impl Parameters {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn new_with_map(map: HashMap<String, serde_json::Value>) -> Self {
        Self { map }
    }
}
