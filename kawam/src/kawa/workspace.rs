use std::{cell::{Ref, RefCell}, collections::HashMap, rc::Rc};

use crate::kawa::window::{Window, WindowId};

pub type WorkspaceId = usize;


pub struct Workspace {
    pub id: WorkspaceId,
    pub windows: HashMap<WindowId, Rc<RefCell<Window>>>,
    pub focused_window: Option<WindowId>,
}

impl Workspace{
    pub fn new(id: WorkspaceId) -> Self{
        Self { id: id, windows: HashMap::new(), focused_window: None }
    }

    //// Windows Operations
    /// Set the focused window inside the workspace
    pub fn set_focused_window(&mut self, id: WindowId) {
        self.focused_window = Some(id);
    }

    /// add a window to the workspace
    pub fn add_window(&mut self, id: WindowId, win : Rc<RefCell<Window>>){
        self.windows.insert(id, win);
    }

    pub fn propose_dimensions_to_windows(&mut self, width:i32, height:i32){
        for (_window_id, window) in &mut self.windows{
            let mut w = window.borrow_mut();
            w.propose_dimensions(width, height);
            drop(w);
        }
    }

}