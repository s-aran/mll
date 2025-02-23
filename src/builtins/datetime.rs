use std::collections::HashMap;
use std::error::Error;

use crate::utils::*;
use crate::Mll;
use chrono::offset::LocalResult;
use chrono::prelude::*;
use mlua::Lua;

use super::builtin::*;

pub struct DateTimeFormatFunction;

#[cfg(feature = "datetime")]
impl BuiltinFunction for DateTimeFormatFunction {
    fn get_name(&self) -> &str {
        "datetime_format"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, (url, data): (String, String)| {
                let c = "";
                let t = HashMap::<&str, &str>::new();

                let r = chitose::sync_http_get(&url, &c, t, &data);
                let r2 = json_str_to_lua_table(&lua_ref, &r);

                Ok(r2.unwrap())
            })
            .unwrap()
    }
}

fn lua_datetime_to_chrono(data: &mlua::Table) -> NaiveDateTime {
    lua_date_to_chrono(data).and_time(lua_time_to_chrono(data))
}

fn lua_date_to_chrono(data: &mlua::Table) -> NaiveDate {
    let year = data.get::<i32>("year").unwrap_or(1970);
    let month = data.get::<u32>("month").unwrap_or(1);
    let day = data.get::<u32>("day").unwrap_or(1);

    let result = NaiveDate::from_ymd_opt(year, month, day).unwrap();

    result
}

fn lua_time_to_chrono(data: &mlua::Table) -> NaiveTime {
    let hour = data.get::<u32>("hour").unwrap_or(0);
    let min = data.get::<u32>("min").unwrap_or(0);
    let sec = data.get::<u32>("sec").unwrap_or(0);

    let result = NaiveTime::from_hms_opt(hour, min, sec).unwrap();

    result
}

fn chrono_datetime_to_lua(lua: &mlua::Lua, data: &NaiveDateTime) -> mlua::Table {
    let table = lua.create_table().unwrap();

    let date = chrono_date_to_lua(lua, &data.date());
    let time = chrono_time_to_lua(lua, &data.time());

    table.set("year", date.get::<i32>("year").unwrap());

    table
}

fn chrono_date_to_lua(lua: &mlua::Lua, data: &NaiveDate) -> mlua::Table {
    let table = lua.create_table().unwrap();

    table.set("year", data.year()).unwrap();
    table.set("month", data.month()).unwrap();
    table.set("day", data.day()).unwrap();

    table
}

fn chrono_time_to_lua(lua: &mlua::Lua, data: &NaiveTime) -> mlua::Table {
    let table = lua.create_table().unwrap();

    table.set("hour", data.hour()).unwrap();
    table.set("min", data.minute()).unwrap();
    table.set("sec", data.second()).unwrap();

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_format() {
        let template = "";

        let lua = Lua::new();
        lua.load(r#"now = os.date("*t")"#).exec().unwrap();

        // print
        for pair in lua
            .globals()
            .get::<mlua::Table>("now")
            .unwrap()
            .pairs::<String, mlua::Value>()
        {
            let (key, value) = pair.unwrap();
            println!("{}: {:?}", key, value);
        }

        let mut mll = Mll::new();
        // mll.set_template(template.to_string());
        let rendered = mll.render(&lua.globals());
        assert_eq!("Hello, hoge!", rendered.unwrap());

        // let tags = mll.get_rendered_tags();
        // assert!(tags.contains(&"name".to_string()));
        // assert_eq!(1, tags.len());
    }
}
