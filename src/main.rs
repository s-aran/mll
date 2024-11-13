use clap::{arg, command, Parser};
use liquid::*;
use std::{collections::HashMap, path::PathBuf};

fn main() {
    println!("Hello, world!");

    let args = Args::parse();

    if !args.is_template_passed() {
        eprintln!("template file is not passed");
        return;
    }

    println!("load {}", args.get_template_file_path().display());

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

    let obj = liquid::object!({
        "name": "hogetaro",
    });

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

    #[arg()]
    templ: Option<String>,

    #[arg(help = "parameters (key=value ...)")]
    params: Vec<String>,
}

impl Args {
    pub fn is_template_passed(&self) -> bool {
        (self.template.is_some() && self.template.as_ref().unwrap() != "")
            || (self.templ.is_some() && self.templ.as_ref().unwrap() != "")
    }

    pub fn is_parameter_passed(&self) -> bool {
        self.parameter.is_some() && self.parameter.as_ref().unwrap() != ""
    }

    pub fn is_output_passed(&self) -> bool {
        self.output.is_some() && self.output.as_ref().unwrap() != ""
    }

    pub fn is_parameters_passed(&self) -> bool {
        self.params.len() > 0
    }

    pub fn get_template_file_path(&self) -> PathBuf {
        match self.templ {
            Some(ref templ) => PathBuf::from(templ.clone()),
            None => PathBuf::from(self.template.clone().unwrap()),
        }
    }

    pub fn get_parameter_file_path(&self) -> PathBuf {
        PathBuf::from(self.parameter.clone().unwrap())
    }

    pub fn get_output_file_path(&self) -> PathBuf {
        PathBuf::from(self.output.clone().unwrap())
    }
}

struct Parameter {
    key: String,
    value: String,
}

struct Parameters {
    map: HashMap<String, String>,
}

impl Parameters {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
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

    pub fn new_from_key_value_params(params: Vec<String>) -> Self {
        let mut map = HashMap::new();

        for p in params.iter() {
            let (k, v) = Parameters::split_key_value(p);
            map.insert(k, v);
        }

        Self { map }
    }
}
