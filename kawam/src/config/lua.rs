use mlua::{Lua, Table,LuaSerdeExt, Result, Function, RegistryKey,Value, IntoLuaMulti};
use tracing::debug;
use std::{cell::RefCell, collections::HashMap, fs, path::{Path, PathBuf}, println, process::{Command, Stdio}, rc::Rc, string};

use serde_json::Value as Json;
use crate::{config::{BindTarget, Binds, Config, Queue, action::LuaAction}, kawa::window::WindowId};
use super::action::{Action};
use super::Handlers;
use super::config_path;

#[derive(Debug)]
pub struct LuaRuntime {
    pub lua: Lua,
    queue: Queue,
    handlers: Handlers,
    binds: Binds,
    pub load_error: Option<mlua::Error>,
}



impl LuaRuntime {
    pub fn new_with(conf_check: bool) -> Result<(Self, Config)>{
        let lua = Lua::new();

        // Global Table
        let kawa = lua.create_table()?;
        // Callbacks 
        let handlers: Handlers = Rc::new(RefCell::new(HashMap::new()));
        let queue : Queue = Rc::new(RefCell::new(Vec::<Action>::new()));
        let binds: Binds = Rc::new(RefCell::new(Vec::<(String, BindTarget)>::new()));
        
        let q = queue.clone();
        let h = handlers.clone();
        let b = binds.clone();   
        
        kawa.set("bind", lua.create_function(move |lua, (keys, target): (String, Value)| {
            let target = match target {
                Value::UserData(ud) => BindTarget::Action(ud.borrow::<LuaAction>()?.0.clone()),
                Value::Function(f) => BindTarget::Callback(lua.create_registry_value(f)?),
                _ => return Err(mlua::Error::runtime(
                    "kawa.bind: need a function or a dispatcher kw.dsp",
                )),
            };
            b.borrow_mut().push((keys, target));
            Ok(())
        })?)?;
        kawa.set(
            "on",
            lua.create_function(move |lua, (event, f): (String, Function)| {
                let key = lua.create_registry_value(f)?;
                h.borrow_mut().entry(event).or_default().push(key);
                Ok(())
            })?,
        )?;


        kawa.set(
            "print",
            lua.create_function(|_, msg: String| {
                println!("[lua] {msg}");
                Ok(())
            })?,
        )?;

        // Exemple of queued action
        kawa.set("close", lua.create_function(move |_, id: WindowId| {
            q.borrow_mut().push(Action::Close(Some(id)));
            Ok(())
        })?)?;

        kawa.set(
            "exec_raw",
            lua.create_function(move |_, cmd: String| {
                if conf_check {
                    debug!("[dry-run] exec_cmd {cmd}");
                    return Ok(());
                }
                let mut args = cmd.split_whitespace();
                let prog = args.next().ok_or_else(|| mlua::Error::runtime("no command"))?;

                let mut child = Command::new(prog)
                    .args(args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map_err(mlua::Error::external)?;

                std::thread::spawn(move || { let _ = child.wait(); });
                Ok(())
            })?,
        )?;

         kawa.set(
            "exec_cmd",
            lua.create_function(move |_, cmd: String| {
                if conf_check {
                    debug!("[dry-run] exec_cmd {cmd}");
                    return Ok(());
                }
                let mut child = Command::new("sh")
                    .arg("-c")
                    .args(cmd.split(" "))
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map_err(mlua::Error::external)?;

                std::thread::spawn(move || { let _ = child.wait(); });
                Ok(())
            })?,
        )?;
        lua.globals().set("kw", kawa.clone())?;
       
        let mut config = Config::default();
        let mut load_error = None;
        let path = config_path();

        match fs::read_to_string(&path) {
            Err(e) => eprintln!("No config file ({}): {e}", path.display()),
            Ok(src) => {
                let mut raw = serde_json::json!({});

                let result = lua.scope(|scope| {
                    kawa.set(
                        "config",
                        scope.create_function_mut(|lua, tbl: Table| {
                            let patch: Json = lua.from_value(Value::Table(tbl))?;

                            let mut candidate = raw.clone();
                            merge(&mut candidate, patch);

                            let parsed: Config = serde_json::from_value(candidate.clone())
                                .map_err(|e| mlua::Error::runtime(format!("kawa.config: {e}")))?;

                            raw = candidate;
                            config = parsed;
                            Ok(())
                        })?,
                    )?;
                    lua.load(&src).set_name(path.to_string_lossy()).exec()
                });

                kawa.set("config", Value::Nil)?;
                load_error = result.err();
            }
        }

        Ok((Self { lua, queue, handlers, binds, load_error }, config))
    }
    
    pub fn new() -> Result<(Self,Config)> {
        Self::new_with(false)
    }
 
    pub fn drop_globals(&self) -> mlua::Result<bool>{
        // backup the globals
        let globals = self.lua.globals();
        let backup: Vec<(Value,Value)> = globals
            .pairs::<Value,Value>()
            .collect::<mlua::Result<_>>()?;

        // Clear and assert clearance
        globals.clear()?;
        let cleared = globals.pairs::<Value, Value>().count() == 0;
        // If the new config have potential bug, restore the previous table and doesn't touch
        // the attribut config in the context
       
        // TODO create a new lua runtime object and see if it pass througth the function new
        let continue_config_swap: bool = match self.check_config(){
            Ok(_) => true,
            Err(_) => false
        }; 
        // Then call <> to give the new config 
        //self.apply_config -> Config 

        if !continue_config_swap && cleared {
            globals.clear()?;
            for (k, v) in &backup {
                globals.raw_set(k.clone(), v.clone())?;
            }
            return Ok(continue_config_swap);
        }
        Ok(continue_config_swap)
    }

    pub fn check_config(&self) -> std::result::Result<(), String> {
        let (rt, _config) = LuaRuntime::new_with(true).map_err(|e| e.to_string())?;
        match rt.load_error {
            Some(e) => Err(format!("error in config file:\n{e}")),
            None => Ok(()),
        }
    }

    pub fn emit<A: IntoLuaMulti + Clone>(&self, event: &str, args: A) {
        let fns: Vec<Function> = self
            .handlers
            .borrow()
            .get(event)
            .into_iter()
            .flatten()
            .filter_map(|k| self.lua.registry_value(k).ok())
            .collect();

        for f in fns {
            if let Err(e) = f.call::<()>(args.clone()) {
                eprintln!("[lua] erreur dans le handler '{event}': {e}");
            }
        }
    }

    pub fn drain(&self) -> Vec<Action> {
        self.queue.borrow_mut().drain(..).collect()
    }

}

/// Merge all the .lua files into a single one
/// Handle the require
fn merge(base: &mut Json, patch: Json) {
    match (base, patch) {
        (Json::Object(b), Json::Object(p)) => {
            for (k, v) in p {
                merge(b.entry(k).or_insert(Json::Null), v);
            }
        }
        (b, p) => *b = p,
    }
}