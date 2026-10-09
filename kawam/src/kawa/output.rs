//! Output management for Kawa Client

use crate::kawa::window;
use crate::protocol::river::{RiverLayerShellOutputV1, RiverOutputV1};
use std::cell::RefCell;
use std::rc::Weak;
use wayland_client::protocol::wl_output::WlOutput;

/// Output identifier
pub type OutputId = usize;

/// A managed output (display)
pub struct Output {
    /// Unique output ID
    pub id: OutputId,
    /// River output protocol object
    pub rwm_output: Option<RiverOutputV1>,
    /// Layer shell output for bar
    pub layer_shell_output: Option<RiverLayerShellOutputV1>,
    /// Wayland output object
    pub wl_output: Option<WlOutput>,
    /// Global name of the wl_output
    pub wl_output_name: u32,

    /// Output name (from wl_output)
    pub name: Option<String>,

    /// Position X in global coordinates
    pub x: i32,
    /// Position Y in global coordinates
    pub y: i32,
    /// Width
    pub width: i32,
    /// Height
    pub height: i32,
    /// Output scale factor (integer)
    pub scale: i32,
    /// Whether this output has been removed
    pub removed: bool,
}

impl Output {
    /// Create a new output
    pub fn new(id: OutputId) -> Self {
        Self {
            id,
            rwm_output: None,
            layer_shell_output: None,
            wl_output: None,
            wl_output_name: 0,
            name: None,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            scale: 1,
            removed: false,
        }
    }

    ///
    pub fn update_dimensions(&mut self, width: i32, height: i32){
        self.width = width;
        self.height = height;
    }

    /// Check if a point is within this output
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Output")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("x", &self.x)
            .field("y", &self.y)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}
