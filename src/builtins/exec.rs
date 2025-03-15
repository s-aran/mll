use std::process::Command;

use mlua::{FromLua, Function, IntoLua, Lua};

use super::builtin::BuiltinFunction;

pub struct Exec;

impl BuiltinFunction for Exec {
    fn get_name(&self) -> &str {
        "exec"
    }

    fn get_function(&self, lua: &Lua) -> Function {
        lua.create_function(|_, (param, args): (String, Vec<String>)| Ok(system(param, args)))
            .unwrap()
    }
}

pub struct ExecResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl IntoLua for ExecResult {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;

        table.set("code", self.code)?;
        table.set("stdout", self.stdout)?;
        table.set("stderr", self.stderr)?;

        Ok(table.into_lua(lua).unwrap())
    }
}

impl FromLua for ExecResult {
    fn from_lua(value: mlua::Value, _: &Lua) -> mlua::Result<Self> {
        let table = value.as_table().unwrap();

        Ok(ExecResult {
            code: table.get("code")?,
            stdout: table.get("stdout")?,
            stderr: table.get("stderr")?,
        })
    }
}

fn system(program: String, args: Vec<String>) -> ExecResult {
    let mut command = Command::new(program.as_str());
    let result = command.args(args);
    let output = result.output().expect("failed to execute process");

    ExecResult {
        code: output.status.code().unwrap(),
        stdout: String::from_utf8(output.stdout)
            .unwrap()
            .trim_end()
            .to_string(),
        stderr: String::from_utf8(output.stderr)
            .unwrap()
            .trim_end()
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::exec::system;

    #[test]
    fn test_system() {
        let program = "rustc".to_string();
        let args = vec!["--version"]
            .iter()
            .map(|e| (*e).into())
            .collect::<Vec<String>>();
        let result = system(program, args);

        assert_eq!(0, result.code);
        assert!(result.stdout.starts_with("rustc"));
        assert!(result.stdout.ends_with(")"));
        assert_eq!("", result.stderr);
    }

    #[test]
    fn test_exec_by_lua() {
        let lua = Lua::new();

        let _ = Exec {}.set_function(&lua);
        let _ = lua.load(r#"result = exec("rustc", {"--version"})"#).exec();

        let result = lua.globals().get::<ExecResult>("result").unwrap();

        assert_eq!(0, result.code);
        assert!(result.stdout.starts_with("rustc"));
        assert!(result.stdout.ends_with(")"));
        assert_eq!("", result.stderr);
    }
}
