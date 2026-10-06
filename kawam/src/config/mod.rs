pub mod lua;
pub mod action;
pub mod dispatcher;
pub mod watcher;

use std::{cell::RefCell, collections::{HashMap, HashSet}, env, path::{Path, PathBuf}, rc::Rc};
use mlua::{Lua, RegistryKey};
use action::Action;
use serde::{self, Deserialize};

pub type Handlers = Rc<RefCell<HashMap<String, Vec<RegistryKey>>>>;
pub type Queue = Rc<RefCell<Vec<Action>>>;
pub type Binds = Rc<RefCell<Vec<(String, BindTarget)>>>;


pub fn config_path() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config")
        });
    base.join("kawa").join("kawa.lua")
}




#[derive(Debug)]
pub enum BindTarget {
    Action(Action),
    Callback(RegistryKey),
}



#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub general: General,
    pub animations: Animations,
    pub scroll_factor: f64,
    pub repeat_delay: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct General {
    pub yes: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Animations { pub enabled: bool }


// Default implementations
impl Default for General{
    fn default() -> Self {
        Self { yes: false }
    }
}


