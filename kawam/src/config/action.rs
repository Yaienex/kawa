use std::cell::RefCell;
use std::rc::Rc;
use crate::kawa::window::WindowId;
use mlua::{UserData, UserDataMethods, MetaMethod};

#[derive(Clone, Debug)]
pub enum Action {
    Exec(String),           
    Close(Option<WindowId>),
    Focus(String),
    ToggleFloat(Option<WindowId>),
    WorkspaceSwitch(u32),
    WorkspaceMoveWindow(u32),
}

#[derive(Clone)]
pub struct LuaAction(pub Action);

impl UserData for LuaAction {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        // print(kawa.dsp.window.close()) -> "Close(None)", debugging 
        m.add_meta_method(MetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this.0)));
    }
}