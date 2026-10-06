pub mod context;
pub mod output;
pub mod seat;
pub mod state;
pub mod window;
pub mod workspace;

use std::collections::HashMap;

// Protocols
use crate::protocol::river::*;
use wayland_client::protocol::wl_compositor;
use wayland_client::protocol::wl_output;
use wayland_client::protocol::wl_seat;
use wayland_client::protocol::wl_shm;
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1;

// Global information of the Wayland Session
#[derive(Default)]
pub struct Globals {
    // The compositor, creator of surfaces
    pub compositor: Option<wl_compositor::WlCompositor>,
    // Shared memory to create buffers
    pub shm: Option<wl_shm::WlShm>,
    // WM client
    pub rwm: Option<RiverWindowManagerV1>,
    // Keybind declaration
    pub rwm_xkb_bindings: Option<RiverXkbBindingsV1>,
    // Extension for River to know the state of the layers shell
    pub rwm_layer_shell: Option<RiverLayerShellV1>,
    // Peripheric management client side
    pub rwm_input_manager: Option<RiverInputManagerV1>,
    // Input config of peripherics
    pub rwm_libinput_config: Option<RiverLibinputConfigV1>,
    // Layer surface ancored to sides
    pub wlr_layer_shell: Option<ZwlrLayerShellV1>,
    // CSM to ask the compositor to change the cursor
    pub cursor_shape_manager: Option<WpCursorShapeManagerV1>,
    // Seats aka peripherical inputs (keyboard, etc)
    pub wl_seats: HashMap<u32, wl_seat::WlSeat>,
    // Seat capability to have a pointer
    pub wl_seat_has_pointer: HashMap<u32, bool>,
    // Map of the ouputs i.e. the screens
    pub wl_outputs: HashMap<u32, wl_output::WlOutput>,
    // Scale factor for each outpur
    pub wl_output_scales: HashMap<u32, i32>,
}
