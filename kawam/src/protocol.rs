//! Implementation of River Protocols binding for Wayland

pub mod river_wm {
    pub extern crate bitflags;
    pub extern crate wayland_backend;
    pub extern crate wayland_client;
    pub use wayland_client::protocol::wl_surface;

    // Generate the description table of the given protocol
    pub mod __interfaces {
        pub use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("./protocols/river-window-management-v1.xml");
    }

    // Generate client side API of the given protocol
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("./protocols/river-window-management-v1.xml");
}

pub mod river_input {
    pub extern crate wayland_backend;
    pub extern crate wayland_client;
    pub use wayland_client::protocol::wl_output;

    // Generate the description table of the given protocol
    pub mod __interfaces {
        pub use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("./protocols/river-input-management-v1.xml");
    }

    // Generate client side API of the given protocol
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("./protocols/river-input-management-v1.xml");
}

pub mod river_xkb_config {
    pub extern crate wayland_backend;
    pub extern crate wayland_client;
    // It depends on this interface
    pub use super::river_input::river_input_device_v1;

    // Generate the description table of the given protocol
    pub mod __interfaces {
        pub use super::super::river_input::__interfaces::*;
        wayland_scanner::generate_interfaces!("./protocols/river-xkb-config-v1.xml");
    }

    // Generate client side API of the given protocol
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("./protocols/river-xkb-config-v1.xml");
}

// Keyboard shortcut binding protocol
pub mod river_xkb {
    pub extern crate bitflags;
    pub extern crate wayland_backend;
    pub extern crate wayland_client;
    pub use super::river_wm::river_seat_v1;

    // Generate the description table of the given protocol
    pub mod __interfaces {
        pub use super::super::river_wm::__interfaces::*;
        wayland_scanner::generate_interfaces!("./protocols/river-xkb-bindings-v1.xml");
    }
    // Generate client side API of the given protocol
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("./protocols/river-xkb-bindings-v1.xml");
}

//Layer Surface protocol
pub mod river_layer_shell {
    pub extern crate wayland_backend;
    pub extern crate wayland_client;

    // needs output and seat
    pub use super::river_wm::{river_output_v1, river_seat_v1};

    // Generate the description table of the given protocol
    pub mod __interfaces {
        pub use super::super::river_wm::__interfaces::*;
        wayland_scanner::generate_interfaces!("./protocols/river-layer-shell-v1.xml");
    }
    // Generate client side API of the given protocol
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("./protocols/river-layer-shell-v1.xml");
}

// River LibInput Config Protocol
pub mod river_libinput_conf {
    use super::river_input::*;
    use wayland_client;

    // Generate the description table of the given protocol
    pub mod __interfaces {
        pub use super::super::river_input::__interfaces::*;
        wayland_scanner::generate_interfaces!("./protocols/river-libinput-config-v1.xml");
    }
    // Generate client side API of the given protocol
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("./protocols/river-libinput-config-v1.xml");
}

// Re export most used types
pub mod river {
    #![allow(unused_imports)]
    use super::*;

    // Window Management Common Types
    pub use river_wm::river_decoration_v1::RiverDecorationV1;
    pub use river_wm::river_node_v1::RiverNodeV1;
    pub use river_wm::river_output_v1::RiverOutputV1;
    pub use river_wm::river_pointer_binding_v1::RiverPointerBindingV1;
    pub use river_wm::river_seat_v1::RiverSeatV1;
    pub use river_wm::river_shell_surface_v1::RiverShellSurfaceV1;
    pub use river_wm::river_window_manager_v1::RiverWindowManagerV1;
    pub use river_wm::river_window_v1::RiverWindowV1;

    // XKB Bindings Common Types
    pub use river_xkb::river_xkb_binding_v1::RiverXkbBindingV1;
    pub use river_xkb::river_xkb_bindings_v1::RiverXkbBindingsV1;

    // Layer Shell Common Types
    pub use river_layer_shell::river_layer_shell_output_v1::RiverLayerShellOutputV1;
    pub use river_layer_shell::river_layer_shell_seat_v1::RiverLayerShellSeatV1;
    pub use river_layer_shell::river_layer_shell_v1::RiverLayerShellV1;

    // Input Common Types
    pub use river_input::river_input_device_v1::RiverInputDeviceV1;
    pub use river_input::river_input_manager_v1::RiverInputManagerV1;

    pub use river_libinput_conf::river_libinput_accel_config_v1::RiverLibinputAccelConfigV1;
    pub use river_libinput_conf::river_libinput_config_v1::RiverLibinputConfigV1;
    pub use river_libinput_conf::river_libinput_device_v1::RiverLibinputDeviceV1;
    pub use river_libinput_conf::river_libinput_result_v1::RiverLibinputResultV1;
}

// Re export Event Enum
pub mod event {
    #![allow(unused_imports)]
    use super::*;

    // Window Manager Event Enum
    pub use river_wm::river_decoration_v1::Event as DEvent;
    pub use river_wm::river_node_v1::Event as NEvent;
    pub use river_wm::river_output_v1::Event as OEvent;
    pub use river_wm::river_pointer_binding_v1::Event as PBEvent;
    pub use river_wm::river_seat_v1::Event as SEvent;
    pub use river_wm::river_shell_surface_v1::Event as ShSEvent;
    pub use river_wm::river_window_manager_v1::Event as WmEvent;
    pub use river_wm::river_window_v1::Event as WEvent;

    pub use river_xkb::river_xkb_binding_v1::Event as BEvent;
    pub use river_xkb::river_xkb_bindings_v1::Event as BsEvent;

    // Layer Shell Event Enum
    pub use river_layer_shell::river_layer_shell_output_v1::Event as LsOEvent;
    pub use river_layer_shell::river_layer_shell_seat_v1::Event as LsSEvent;
    pub use river_layer_shell::river_layer_shell_v1::Event as LsEvent;

    // Input Event Enum
    pub use river_input::river_input_device_v1::Event as IDEvent;
    pub use river_input::river_input_manager_v1::Event as IMEvent;
}
