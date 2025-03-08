use chrono::Duration;
use chrono::prelude::*;
use mlua::Lua;
use mlua::Table;

use super::builtin::*;

pub struct DateTimeFormat;

impl BuiltinFunction for DateTimeFormat {
    fn get_name(&self) -> &str {
        "datetime_format"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, (datetime, format): (Table, String)| {
                let dt = lua_datetime_to_chrono(&datetime);
                let formatted = dt.format(&format);
                Ok(formatted.to_string())
            })
            .unwrap()
    }
}

pub struct DateTimeOffset;

impl BuiltinFunction for DateTimeOffset {
    fn get_name(&self) -> &str {
        "datetime_offset"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(
                move |_,
                      (datetime, weeks, days, hours, minutes, seconds): (
                    Table,
                    Option<i64>,
                    Option<i64>,
                    Option<i64>,
                    Option<i64>,
                    Option<i64>,
                )| {
                    let weeks = Duration::weeks(weeks.unwrap_or(0));
                    let days = Duration::days(days.unwrap_or(0));
                    let hours = Duration::hours(hours.unwrap_or(0));
                    let minutes = Duration::minutes(minutes.unwrap_or(0));
                    let seconds = Duration::seconds(seconds.unwrap_or(0));

                    let dt = lua_datetime_to_chrono(&datetime)
                        + weeks
                        + days
                        + hours
                        + minutes
                        + seconds;

                    Ok(chrono_datetime_to_lua(&lua_ref, &dt))
                },
            )
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

    let _ = table.set("year", date.get::<i32>("year").unwrap_or(1970));
    let _ = table.set("month", date.get::<i32>("month").unwrap_or(1));
    let _ = table.set("day", date.get::<i32>("day").unwrap_or(1));

    let _ = table.set("hour", time.get::<i32>("hour").unwrap_or(0));
    let _ = table.set("min", time.get::<i32>("min").unwrap_or(0));
    let _ = table.set("sec", time.get::<i32>("sec").unwrap_or(0));

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
    use crate::Mll;

    #[test]
    fn test_datetime_format() {
        let template = "{{formatted_datetime}}";
        let mut mll = Mll::new();
        mll.set_template(template.to_owned());
        mll.set_pre_process_script(
            r#"
            local datetime = {
                year = 2020,
                month = 1,
                day = 2,
                hour = 12,
                min = 34,
                sec = 56
            };

            formatted_datetime = datetime_format(datetime, "%Y年%m月%d日　%H時%M分%S秒");
        "#
            .to_string(),
        );

        let rendered = mll.render_lua_globals();
        let expected = "2020年01月02日　12時34分56秒";
        assert_eq!(expected, rendered.unwrap());
    }

    #[test]
    fn test_datetime_offset() {
        let template = "{{formatted_datetime}}";
        let mut mll = Mll::new();
        mll.set_template(template.to_owned());
        mll.set_pre_process_script(
            r#"
            local datetime = {
                year = 2020,
                month = 1,
                day = 2,
                hour = 12,
                min = 34,
                sec = 56
            };

            datetime = datetime_offset(datetime, 1, -2, 3, -4, 5);

            formatted_datetime = datetime_format(datetime, "%Y年%m月%d日　%H時%M分%S秒");
        "#
            .to_string(),
        );

        let rendered = mll.render_lua_globals();
        let expected = "2020年01月07日　15時31分01秒";
        assert_eq!(expected, rendered.unwrap());
    }

    #[test]
    fn test_datetime_offset_nil() {
        let template = "{{formatted_datetime}}";
        let mut mll = Mll::new();
        mll.set_template(template.to_owned());
        mll.set_pre_process_script(
            r#"
            local datetime = {
                year = 2020,
                month = 1,
                day = 2,
                hour = 12,
                min = 34,
                sec = 56
            };

            datetime = datetime_offset(datetime, nil, nil, nil, nil, nil);

            formatted_datetime = datetime_format(datetime, "%Y年%m月%d日　%H時%M分%S秒");
        "#
            .to_string(),
        );

        let rendered = mll.render_lua_globals();
        let expected = "2020年01月02日　12時34分56秒";
        assert_eq!(expected, rendered.unwrap());
    }
}
