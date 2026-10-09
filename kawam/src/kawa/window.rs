//! Window Object manage by Kawa Client
//! So we don't manipulate raw Wayland data

use mlua::Debug;

use crate::protocol::{river::{RiverNodeV1, RiverWindowV1}, river_wm::river_window_v1::Edges};


// Window Identifier
pub type WindowId = usize;

pub struct Window {
    pub id: WindowId,
    pub rwm_window: Option<RiverWindowV1>,
    pub rwm_node: Option<RiverNodeV1>,
    pub app_id : Option<String>,
    pub title: Option<String>,


    pub width: i32,
    pub height: i32,
    pub proposed_width: i32,
    pub proposed_height: i32,
}


impl Window{
    pub fn new(id: WindowId) -> Self {
        Self { id:id , 
            rwm_window: None, 
            rwm_node: None,
            app_id: None, 
            title: None,

            width : 0,
            height : 0,
            proposed_width: 0,
            proposed_height: 0,
        }
    }


    /// Propose dimensions for the window
    pub fn propose_dimensions(&mut self, width: i32, height: i32) {
        self.proposed_width = width.max(0);
        self.proposed_height = height.max(0);

        // Send to compositor via protocol
        if let Some(ref rwm_window) = self.rwm_window {
            rwm_window.propose_dimensions(self.proposed_width, self.proposed_height);
        }
    }

    pub fn update_dimensions(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    /// Place this window at top of render list
    pub fn place_top(&self) {
        if let Some(ref node) = self.rwm_node {
            node.place_top();
        }
    }

    pub fn set_borders(&self, edges: Edges, width: i32, r: u32, g: u32, b: u32, a: u32) {
        if let Some(ref rwm_window) = self.rwm_window {
            rwm_window.set_borders(edges, width, r, g, b, a);
        }
    }
}



impl std::fmt::Debug for Window{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("id", &self.id)
            .field("name", &self.title)
            .finish()
    }
}