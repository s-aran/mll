use super::builtin::BuiltinFunction;

trait DatabaseSystemName {
    fn get_name(&self) -> &str;
}

pub enum Connection {
    MySql(my_sql::MySqlConnectionConfig),
    NoDb(no_db::NoDb),
}

impl Default for Connection {
    fn default() -> Self {
        Self::NoDb(no_db::NoDb::default())
    }
}

mod my_sql {
    use mlua::{FromLua, IntoLua, Lua, Table};
    use sqlx::mysql::{
        MySqlColumn, MySqlConnectOptions, MySqlConnection, MySqlPool, MySqlRow, MySqlSslMode,
    };
    use sqlx::{Column, ConnectOptions, Connection, Row};
    use uuid::Uuid;

    use crate::builtins::builtin::BuiltinFunction;
    use crate::utils::do_blocking;

    use super::DatabaseSystemName;

    pub struct ConnectMySql {
        key: String,
        connection: MySqlConnectionConfig,
    }

    impl BuiltinFunction for ConnectMySql {
        fn get_name(&self) -> &str {
            "connect_mysql"
        }

        fn get_function(&self, lua: &Lua) -> mlua::Function {
            let lua_ref = lua.clone();
            lua_ref
                .clone()
                .create_function(move |lua, config: MySqlConnectionConfig| {
                    let uuid = Uuid::new_v4();
                    let key = uuid
                        .simple()
                        .encode_lower(&mut Uuid::encode_buffer())
                        .to_owned();

                    let table = lua.create_table()?;
                    table.set(key.clone(), config)?;
                    lua.globals().set("qlp_internal", table)?;

                    Ok(key)
                })
                .unwrap()
        }
    }

    pub struct MySql;

    impl BuiltinFunction for MySql {
        fn get_name(&self) -> &str {
            "execute_sql"
        }

        fn get_function(&self, lua: &Lua) -> mlua::Function {
            let lua_ref = lua.clone();
            lua_ref
                .clone()
                .create_function(move |lua, (connection_string, query): (String, String)| {
                    let table: Table = lua.globals().get("qlp_internal")?;
                    let connection_data: MySqlConnectionConfig = table.get(connection_string)?;

                    let conn = do_blocking(
                        MySqlConnectOptions::new()
                            .host(connection_data.host.as_str())
                            .port(connection_data.port)
                            .username(connection_data.username.as_str())
                            .password(connection_data.password.as_str())
                            .database(connection_data.database.as_str())
                            .connect(),
                    );

                    if conn.is_err() {
                        return Err(mlua::Error::RuntimeError(
                            "Failed to connect to database".to_string(),
                        ));
                    }

                    let result =
                        do_blocking(sqlx::query(query.as_str()).fetch_all(&mut conn.unwrap()));

                    if result.is_err() {
                        return Err(mlua::Error::RuntimeError(
                            "Failed to execute query".to_string(),
                        ));
                    }

                    let fetched = result.unwrap();
                    let table = lua.create_table()?;

                    for row in fetched {
                        let row_table = lua.create_table()?;

                        for column in row.columns() {
                            row_table.set(column.name(), 0)?;
                        }

                        table.push(row_table)?;
                    }

                    return Ok(90);
                })
                .unwrap()
        }
    }

    pub struct MySqlConnectionConfig {
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
    }

    impl IntoLua for MySqlConnectionConfig {
        fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
            let table = lua.create_table()?;

            table.set("host", self.host)?;
            table.set("port", self.port)?;
            table.set("database", self.database)?;
            table.set("username", self.username)?;
            table.set("password", self.password)?;

            Ok(table.into_lua(lua).unwrap())
        }
    }

    impl FromLua for MySqlConnectionConfig {
        fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
            let table = value.as_table().unwrap();

            Ok(Self {
                host: table.get("host")?,
                port: table.get("port")?,
                database: table.get("database")?,
                username: table.get("username")?,
                password: table.get("password")?,
            })
        }
    }

    impl Default for MySqlConnectionConfig {
        fn default() -> Self {
            Self {
                host: "localhost".to_string(),
                port: 3306,
                database: "".to_string(),
                username: "".to_string(),
                password: "".to_string(),
            }
        }
    }

    impl DatabaseSystemName for MySqlConnectionConfig {
        fn get_name(&self) -> &str {
            "mysql"
        }
    }

    fn execute_sql_for_mysql() {
        let conn = MySqlConnectOptions::new()
            .host("localhost")
            .port(3306)
            .database("test")
            .username("root")
            .password("root");
    }
}

mod no_db {
    use super::DatabaseSystemName;

    #[derive(Default)]
    pub struct NoDb {}

    impl DatabaseSystemName for NoDb {
        fn get_name(&self) -> &str {
            ""
        }
    }
}
