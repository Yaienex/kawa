//! Output management for Kawa Client

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

    /// Currently fullscreen window on this output
    pub fullscreen_window: Option<Weak<RefCell<super::window::Window>>>,

    /// Exclusive area (for layer shell surfaces like bars)
    pub exclusive_x: i32,
    pub exclusive_y: i32,
    pub exclusive_width: i32,
    pub exclusive_height: i32,

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
            fullscreen_window: None,
            exclusive_x: 0,
            exclusive_y: 0,
            exclusive_width: 0,
            exclusive_height: 0,
            removed: false,
        }
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
