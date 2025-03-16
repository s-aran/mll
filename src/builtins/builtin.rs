use mlua::Lua;

/// A trait for defining a built-in function
///
/// # Example
///
/// ```
/// use mlua::Lua;
/// use mlua::prelude::*;
///
/// use mll::builtins::builtin::*;
///
/// pub struct MyFunction;
///
/// impl BuiltinFunction for MyFunction {
///     fn get_name(&self) -> &str {
///         "my_function"
///     }
///
///     fn get_function(&self, lua: &Lua) -> mlua::Function {
///         let lua_ref = lua.clone();
///         lua_ref
///             .clone()
///             .create_function(|_, ()| {
///                 Ok("Hello, World!")
///        })
///        .unwrap()
///     }
/// }
/// ```
pub trait BuiltinFunction {
    /// Get the name of the function
    ///
    /// # Returns
    ///
    /// `&str` - The name of the function
    fn get_name(&self) -> &str;

    /// Get the function
    ///
    /// # Arguments
    ///
    /// * `lua` - The Lua context
    ///
    /// # Returns
    ///
    /// `mlua::Function` - The function
    fn get_function(&self, lua: &Lua) -> mlua::Function;

    fn set_function(&self, lua: &Lua) -> mlua::Result<()> {
        let globals = lua.globals();
        let name = self.get_name().to_owned();
        let func = self.get_function(lua);
        globals.set(name, func)?;
        Ok(())
    }
}
