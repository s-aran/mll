//! HTTP request commands.

use core::panic;
use std::collections::HashMap;

use chitose::method::HttpMethod;
use mlua::{FromLua, IntoLua, Lua, Table};

use crate::utils::json_str_to_lua_table;

use super::builtin::*;

pub struct SimpleHttpGet;

impl BuiltinFunction for SimpleHttpGet {
    fn get_name(&self) -> &str {
        "simple_http_get"
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

pub struct SimpleHttpPost;

impl BuiltinFunction for SimpleHttpPost {
    fn get_name(&self) -> &str {
        "simple_http_post"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, (url, data): (String, String)| {
                let c = "";
                let t = HashMap::<&str, &str>::new();

                let r = chitose::sync_http_post(&url, &c, t, &data);
                let r2 = json_str_to_lua_table(&lua_ref, &r);

                Ok(r2.unwrap())
            })
            .unwrap()
    }
}

pub struct SimpleHttpPut;

impl BuiltinFunction for SimpleHttpPut {
    fn get_name(&self) -> &str {
        "simple_http_put"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, (url, data): (String, String)| {
                let c = "";
                let t = HashMap::<&str, &str>::new();

                let r = chitose::sync_http_put(&url, &c, t, &data);
                let r2 = json_str_to_lua_table(&lua_ref, &r);

                Ok(r2.unwrap())
            })
            .unwrap()
    }
}

pub struct SimpleHttpDelete;

impl BuiltinFunction for SimpleHttpDelete {
    fn get_name(&self) -> &str {
        "simple_http_delete"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, (url, data): (String, String)| {
                let c = "";
                let t = HashMap::<&str, &str>::new();

                let r = chitose::sync_http_delete(&url, &c, t, &data);
                let r2 = json_str_to_lua_table(&lua_ref, &r);

                Ok(r2.unwrap())
            })
            .unwrap()
    }
}

pub(crate) struct MllHttpMethod(HttpMethod);

impl From<String> for MllHttpMethod {
    fn from(value: String) -> Self {
        match value.to_ascii_uppercase().as_str() {
            "GET" => MllHttpMethod(HttpMethod::GET),
            "POST" => MllHttpMethod(HttpMethod::POST),
            "PUT" => MllHttpMethod(HttpMethod::PUT),
            "DELETE" => MllHttpMethod(HttpMethod::DELETE),
            _ => panic!("Unknown http method"),
        }
    }
}

impl Default for MllHttpMethod {
    fn default() -> Self {
        MllHttpMethod(HttpMethod::GET)
    }
}

#[derive(Default)]
pub struct HttpRequest {
    url: String,
    method: MllHttpMethod,
    header: HashMap<String, String>,
    body: String,
}

impl HttpRequest {
    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn method(&self) -> &MllHttpMethod {
        &self.method
    }

    pub fn header(&self) -> &HashMap<String, String> {
        &self.header
    }

    pub fn body(&self) -> &String {
        &self.body
    }
}

impl ToString for MllHttpMethod {
    fn to_string(&self) -> String {
        match self.0 {
            HttpMethod::GET => "GET".to_string(),
            HttpMethod::POST => "POST".to_string(),
            HttpMethod::PUT => "PUT".to_string(),
            HttpMethod::DELETE => "DELETE".to_string(),
        }
    }
}

impl FromLua for HttpRequest {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        let table = value.as_table().unwrap();
        let url = table.get::<String>("url").unwrap().as_str().to_string();
        let method = table
            .get::<String>("method")
            .unwrap()
            .as_str()
            .to_string()
            .into();
        let header = table
            .get::<Table>("header")
            .unwrap()
            .pairs::<mlua::String, mlua::String>()
            .into_iter()
            .map(|p| match p {
                Ok((k, v)) => (
                    k.to_str().unwrap().to_string(),
                    v.to_str().unwrap().to_string(),
                ),
                Err(_) => panic!("Error parsing header"),
            })
            .collect();
        let body = table.get::<String>("body").unwrap().to_string();

        Ok(Self {
            url,
            method,
            header,
            body,
        })
    }
}

impl IntoLua for HttpRequest {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;
        table.set("url", self.url)?;
        table.set("method", self.method.to_string())?;
        table.set("header", self.header)?;
        table.set("body", self.body)?;

        Ok(mlua::Value::Table(table))
    }
}

pub struct SendHttpRequest;

impl BuiltinFunction for SendHttpRequest {
    fn get_name(&self) -> &str {
        "send_http_request"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, request: HttpRequest| {
                let header = request
                    .header()
                    .into_iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                let r = match request.method().0 {
                    HttpMethod::GET => {
                        chitose::sync_http_get(request.url(), "", header, request.body())
                    }
                    HttpMethod::POST => {
                        chitose::sync_http_post(request.url(), "", header, request.body())
                    }
                    HttpMethod::PUT => {
                        chitose::sync_http_put(request.url(), "", header, request.body())
                    }
                    HttpMethod::DELETE => {
                        chitose::sync_http_delete(request.url(), "", header, request.body())
                    }
                };

                let r2 = json_str_to_lua_table(&lua_ref, &r);

                Ok(r2.unwrap())
            })
            .unwrap()
    }
}
