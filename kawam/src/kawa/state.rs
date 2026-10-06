use std::cell::RefCell;
use std::rc::Rc;

use tracing::{debug, info};
use wayland_client;
use wayland_client::protocol::{
    wl_buffer, wl_compositor, wl_output, wl_registry, wl_seat, wl_shm, wl_shm_pool,
};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::{
    self, WpCursorShapeManagerV1,
};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::{self, ZwlrLayerShellV1};

use super::Globals;
use super::context::Context;
use crate::kawa::seat::Seat;
use crate::protocol::event;
use crate::protocol::river::*;
use crate::protocol::river_input::{river_input_device_v1};
use crate::protocol::river_layer_shell::river_layer_shell_v1;
use crate::protocol::river_libinput_conf::{
    river_libinput_config_v1, river_libinput_device_v1, river_libinput_result_v1,
};
use crate::protocol::river_wm::river_window_manager_v1;
use crate::protocol::river_xkb::river_xkb_bindings_v1;

pub struct AppState {
    pub context: Rc<RefCell<Context>>,
    pub globals: Globals,
}

// Helpers
fn attach_wl_seat_to_kawa_seat(state: &AppState, kawa_seat: Rc<RefCell<Seat>>) {
    let wl_seat_name = kawa_seat.borrow().wl_seat_name;
    if wl_seat_name == 0 || kawa_seat.borrow().wl_seat.is_some() {
        return;
    }
    let wl_seat = match state.globals.wl_seats.get(&wl_seat_name) {
        Some(seat) => seat.clone(),
        None => return,
    };

    {
        let mut seat = kawa_seat.borrow_mut();
        seat.wl_seat = Some(wl_seat.clone());
    }
}

// Dispatch to handle River/Wayland Events

// Listen to Wl Registry and search for global interfaces (Wayland or River)
impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _connhandle: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
            ..
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    let compositor: wl_compositor::WlCompositor =
                        registry.bind(name, version.min(4), qh, ());
                    state.globals.compositor = Some(compositor);
                }
                "wl_shm" => {
                    let shm: wl_shm::WlShm = registry.bind(name, version.min(1), qh, ());
                    state.globals.shm = Some(shm);
                }
                "river_window_manager_v1" => {
                    let rwm: RiverWindowManagerV1 = registry.bind(name, version.min(4), qh, ());
                    state.globals.rwm = Some(rwm.clone());
                    state.context.borrow_mut().rwm = Some(rwm);
                }
                "river_xkb_bindings_v1" => {
                    let xkb: RiverXkbBindingsV1 = registry.bind(name, version.min(4), qh, ());
                    state.globals.rwm_xkb_bindings = Some(xkb.clone());
                    state.context.borrow_mut().rwm_xkb_bindings = Some(xkb);
                }
                "river_layer_shell_v1" => {
                    let rls: RiverLayerShellV1 = registry.bind(name, version.min(4), qh, ());
                    state.globals.rwm_layer_shell = Some(rls.clone());
                    state.context.borrow_mut().rwm_layer_shell = Some(rls);
                }
                "river_input_manager_v1" => {
                    let rim: RiverInputManagerV1 = registry.bind(name, version.min(4), qh, ());
                    state.globals.rwm_input_manager = Some(rim.clone());
                    state.context.borrow_mut().rwm_input_manager = Some(rim);
                }
                "river_libinput_config_v1" => {
                    let config: RiverLibinputConfigV1 = registry.bind(name, version.min(4), qh, ());
                    state.globals.rwm_libinput_config = Some(config.clone());
                    state.context.borrow_mut().rwm_libinput_config = Some(config);
                }
                "zwlr_layer_shell_v1" => {
                    let ls: ZwlrLayerShellV1 = registry.bind(name, version.min(4), qh, ());
                    state.globals.wlr_layer_shell = Some(ls);
                }
                "wp_cursor_shape_manager_v1" => {
                    let manager: WpCursorShapeManagerV1 =
                        registry.bind(name, version.min(2), qh, ());
                    state.globals.cursor_shape_manager = Some(manager);
                }
                "wl_seat" => {
                    let seat: wl_seat::WlSeat = registry.bind(name, version.min(7), qh, name);
                    state.globals.wl_seats.insert(name, seat);
                    state.globals.wl_seat_has_pointer.insert(name, false);

                    let seats: Vec<_> = state.context.borrow().seats.values().cloned().collect();
                    // binding the wl_seat to kawa Seat
                    for seat_ref in seats {
                        if seat_ref.borrow().wl_seat_name == name {
                            attach_wl_seat_to_kawa_seat(state, seat_ref);
                            break;
                        }
                    }
                }
                "wl_output" => {
                    let output: wl_output::WlOutput = registry.bind(name, version.min(4), qh, ());
                    state.globals.wl_outputs.insert(name, output.clone());
                    state.globals.wl_output_scales.entry(name).or_insert(1);

                    let outputs: Vec<_> =
                        state.context.borrow().outputs.values().cloned().collect();

                    for output_ref in outputs {
                        let mut out = output_ref.borrow_mut();
                        // retrieving the correct output
                        if out.wl_output_name != name {
                            continue;
                        }
                        // collecting the wl_output of the output
                        out.wl_output = Some(output.clone());
                        out.scale = state
                            .globals
                            .wl_output_scales
                            .get(&name)
                            .copied()
                            .unwrap_or(1);
                        // drop the borrow on the output
                        drop(out);
                    }
                }

                _ => {}
            }
        }
    }
}

impl Dispatch<RiverWindowManagerV1, ()> for AppState {
    fn event(
        state: &mut Self,
        rwm: &RiverWindowManagerV1,
        event: event::WmEvent,
        _data: &(),
        _connhandle: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use crate::protocol::event::WmEvent;
        match event {
            WmEvent::Unavailable => {
                info!("Event Unavailable");
                info!("Termination of the current Client");
                state.context.borrow_mut().request_stop = true;
            }
            WmEvent::Finished => {
                info!("Event Finished");
                info!("Termination of the current Client");
                state.context.borrow_mut().request_stop = true;
            }
            WmEvent::ManageStart => {
                debug!("Event Manage Start");
                state.context.borrow_mut().handle_manage_start();
                state.context.borrow().finish_manage();
                debug!("Event Manage Finish");
            }
            WmEvent::RenderStart => {
                debug!("Event Render Start");
                state.context.borrow().finish_render();
                debug!("Event Render finish");
            }
            WmEvent::SessionLocked => {
                info!("Event Session Locked");
            }
            WmEvent::SessionUnlocked => {
                info!("Event Session Unlocked");
            }
            WmEvent::Window { id } => {
                info!("A new window has been created with id : {:?}", id.id());
            }
            WmEvent::Output { id } => {
                info!("A new output has been plugged with id : {:?}", id.id());
            }
            WmEvent::Seat { id } => {
                info!("A new seat has been created with id : {:?}", id.id());
            }
        }
    }

    fn event_created_child(
        opcode: u16,
        qhandle: &QueueHandle<Self>,
    ) -> std::sync::Arc<dyn wayland_backend::client::ObjectData> {
        match opcode {
            // input_device event (opcode 1) creates a river_input_device_v1
            river_window_manager_v1::EVT_WINDOW_OPCODE => qhandle.make_data::<RiverWindowV1, _>(()),
            river_window_manager_v1::EVT_OUTPUT_OPCODE => qhandle.make_data::<RiverOutputV1, _>(()),
            river_window_manager_v1::EVT_SEAT_OPCODE => qhandle.make_data::<RiverSeatV1, _>(()),
            _ => unreachable!("unknown event opcode {}", opcode),
        }
    }
}

impl Dispatch<RiverWindowV1, ()> for AppState {
    fn event(
        state: &mut Self,
        window: &RiverWindowV1,
        event: event::WEvent,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use crate::protocol::event::WEvent;
        // first find the window inside our windows map inside our context
        let context = state.context.borrow();

        drop(context);

        match event {
            WEvent::Title { title } => {}

            _ => {}
        }
    }
}

impl Dispatch<RiverSeatV1, ()> for AppState {
    fn event(
        state: &mut Self,
        seat: &RiverSeatV1,
        event: event::SEvent,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        // First, find the seat we are actually managing
        let context = state.context.borrow();

        drop(context);
    }
}

impl Dispatch<RiverOutputV1, ()> for AppState {
    fn event(
        state: &mut Self,
        output: &RiverOutputV1,
        event: event::OEvent,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        // First, find the output we are actually managing
        let context = state.context.borrow();
        info!("here comme the output : {}", output.id());
        drop(context);
    }
}

impl Dispatch<RiverInputManagerV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _rim: &RiverInputManagerV1,
        event: event::IMEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // A new device has been created
        if let event::IMEvent::InputDevice { id } = event {
            //  Configure it
            //TODO adapt when config will be bigger

        }
    }

    fn event_created_child(
        opcode: u16,
        qhandle: &QueueHandle<Self>,
    ) -> std::sync::Arc<dyn wayland_backend::client::ObjectData> {
        match opcode {
            // input_device event (opcode 1) creates a river_input_device_v1
            1 => qhandle.make_data::<RiverInputDeviceV1, _>(()),
            _ => unreachable!("unknown event opcode {}", opcode),
        }
    }
}

impl Dispatch<RiverInputDeviceV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _rid: &RiverInputDeviceV1,
        _event: river_input_device_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        // Input Device Events
        // Could response to some of InDev Event
    }
}

impl Dispatch<wl_output::WlOutput, ()> for AppState {
    fn event(
        state: &mut Self,
        output: &wl_output::WlOutput,
        event: wl_output::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Scale { factor } = event {
            let scale = factor.max(1);
            let output_name = state
                .globals
                .wl_outputs
                .iter()
                .find_map(|(name, output_it)| {
                    if output_it == output {
                        Some(*name)
                    } else {
                        None
                    }
                });
            if let Some(name) = output_name {
                state.globals.wl_output_scales.insert(name, scale);
            }
            let outputs = {
                let context = state.context.borrow();
                context
                    .outputs
                    .values()
                    .filter(|output_it| {
                        let out = output_it.borrow();
                        if let Some(name) = output_name {
                            out.wl_output_name == name
                        } else {
                            out.wl_output.as_ref().map(|o| o == output).unwrap_or(false)
                        }
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            };
            for output_it in outputs {
                output_it.borrow_mut().scale = scale;
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, u32> for AppState {
    fn event(
        state: &mut Self,
        _wl_seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        seat_name: &u32,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wl_seat::Event::Capabilities { capabilities } => {
                let has_pointer = matches!(
                    capabilities,
                    wayland_client::WEnum::Value(caps)
                        if caps.contains(wl_seat::Capability::Pointer)
                );
                state
                    .globals
                    .wl_seat_has_pointer
                    .insert(*seat_name, has_pointer);
            }

            _ => {}
        }
    }
}

/// Lib Input
// Implement dispatch for Libinput Config
impl Dispatch<RiverLibinputConfigV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &RiverLibinputConfigV1,
        _event: river_libinput_config_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Libinput config events
    }

    fn event_created_child(
        opcode: u16,
        qhandle: &QueueHandle<Self>,
    ) -> std::sync::Arc<dyn wayland_client::backend::ObjectData> {
        match opcode {
            // libinput_device event (opcode 1) creates a river_libinput_device_v1
            river_libinput_config_v1::EVT_LIBINPUT_DEVICE_OPCODE => {
                qhandle.make_data::<RiverLibinputDeviceV1, _>(())
            }
            _ => unreachable!("unknown event opcode {}", opcode),
        }
    }
}

impl Dispatch<RiverLibinputDeviceV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &RiverLibinputDeviceV1,
        _event: river_libinput_device_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        // Libinput Device Events
    }

    fn event_created_child(
        _opcode: u16,
        qhandle: &QueueHandle<Self>,
    ) -> std::sync::Arc<dyn wayland_client::backend::ObjectData> {
        qhandle.make_data::<RiverLibinputResultV1, _>(())
    }
}

impl Dispatch<RiverLibinputResultV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &RiverLibinputResultV1,
        _event: river_libinput_result_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Libinput config result events
    }
}

//// No Events Dispatchs
impl Dispatch<ZwlrLayerShellV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrLayerShellV1,
        _event: zwlr_layer_shell_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // zwlr_layer_shell_v1 has no events.
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // wl_compositor has no events
    }
}

impl Dispatch<wl_shm::WlShm, ()> for AppState {
    fn event(
        _state: &mut Self,
        _shm: &wl_shm::WlShm,
        event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_shm::Event::Format { format: _ } = event {
            // We only need ARGB8888 which is always available
        }
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // wl_shm_pool has no events
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_buffer::Event::Release = event {
            // Buffer is no longer in use by compositor
        }
    }
}

impl Dispatch<RiverXkbBindingsV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _rxkbb: &RiverXkbBindingsV1,
        _event: river_xkb_bindings_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // XKB bindings global events
    }
}

impl Dispatch<RiverLayerShellV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _rls: &RiverLayerShellV1,
        _event: river_layer_shell_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // River layer shell Globals Event
    }
}

impl Dispatch<WpCursorShapeManagerV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WpCursorShapeManagerV1,
        _event: wp_cursor_shape_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Cursor shape manager has no events.
    }
}
