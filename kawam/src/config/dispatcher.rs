use mlua::{FromLuaMulti, Function, Lua, Result, Table};

use crate::config::action::LuaAction;

use super::Action;

fn dispatcher<A, F>(lua: &Lua, make: F) -> Result<Function>
where
    A: FromLuaMulti,
    F: Fn(A) -> Action + 'static,
{
    lua.create_function(move |_, args: A| Ok(LuaAction(make(args))))
}