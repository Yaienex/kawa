//! Kawa - River Window Manager in Rust
//!
//! A window manager for the River Wayland compositor.

// The Windows Manager in itself
mod kawa;
// All the implementation of River Protocol made readable in Rust
mod protocol;
// Config of the WM with hot reload
mod config;
// IPC Server for kawactl
mod ipc;

use std::cell::RefCell;
use std::os::unix::io::{AsFd, AsRawFd};
use std::path::PathBuf;
use std::rc::Rc;
use std::{env, fs};
use inotify::{EventMask, Inotify, WatchMask};
use tracing::{debug, error, info, warn};
use wayland_client::{Connection, EventQueue};

use crate::config::watcher::ConfigWatcher;
use crate::ipc::IpcServer;
use crate::kawa::Globals;
use crate::kawa::context::Context;
use crate::kawa::state::AppState;
use crate::config::lua;

pub fn socket_path() -> PathBuf {
    let dir = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    dir.join("kawa.sock")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Connect to Wayland Display
    let conn = Connection::connect_to_env().expect("Please run in River environment");
    let display = conn.display();

    // Create the event queue
    let mut event_queue: EventQueue<AppState> = conn.new_event_queue();
    let qh = event_queue.handle();

    // Fetch Config  and Create Lua Handler
    let (lua,config) = lua::LuaRuntime::new().expect("init Lua failed");

    // Create context
    let ctx = Context::new(lua, config).expect("Could not created context");
    let context = Rc::new(RefCell::new(ctx));

    // Create Conf file watcher
    let mut config_watcher = ConfigWatcher::new().expect("Could not create config watcher");


    // Create App
    let mut state = AppState {
        context: context.clone(),
        globals: Globals::default(),
    };
    let mut ipc_server = IpcServer::bind().expect("Could not bind IPC server");

    // Get Registry and collect global
    // Call the Dispatch WlRegistry multiples times with the differents Globals Variables
    let _registry = display.get_registry(&qh, ());
    event_queue.roundtrip(&mut state)?;

    // Check if the River WM is set and the compositor too
    if state.globals.rwm.is_none() {
        return Err("River window manager protocol not available".into());
    }
    if state.globals.compositor.is_none() {
        return Err("Wayland Compositor is not set, are you running this on Wayland ?".into());
    }
    
    info!("Main Loop Start");
    // emit a signal so the corresponding kw.on do its job
    state.context.borrow().lua.emit("kawa.start", ());
    while !state.context.borrow().request_stop {
        // Flush Out Going Request
        conn.flush()?;

        // Prepare poll file descriptors
        let wayland_fd = conn.as_fd().as_raw_fd();
        let ipc_fd = ipc_server.raw_fd();
        let watcher_fd = config_watcher.as_raw_fd();

        let mut poll_fds = vec![
            libc::pollfd {
                fd: wayland_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: ipc_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd { 
                fd: watcher_fd, 
                events: libc::POLLIN, 
                revents: 0
            },
        ];
        // Telling to Wayland we are going to read events
        // None => Events are waiting
        let guard = if let Some(g) = event_queue.prepare_read() {
            g
        } else {
            continue;
        };
        // -1 is infinite sleeping aka freeing the cpu
        //TODO could be use for animation with a timeout depending on animation speed and refresh rate
        let timeout = if state.context.borrow().anim_start_time.is_some() {
            let max_refresh_mhz = 60_000;
            (1_000_000 / max_refresh_mhz.max(1)) as i32
        } else {
            -1
        };

        //TODO could be use for animation with a timeout depending on animation speed and refresh rate
        let ret = unsafe {
            libc::poll(
                poll_fds.as_mut_ptr(),
                poll_fds.len() as libc::nfds_t,
                timeout,
            )
        };

        // Timeout trigger
        if ret == 0 {
            info!("timeout trigger");
            // we need to release the lock on the wayland socket
            // The elapsed time given for animation has passed
            drop(guard);

            // We need to start the next manage sequence as soon as possible
            if let Some(wm) = &state.context.borrow().rwm {
                wm.manage_dirty();
            }
            continue;
        } else if ret > 0 {
            // PollHup and PollErr to check compositor death
            if poll_fds[0].revents & (libc::POLLIN | libc::POLLHUP | libc::POLLERR) != 0 {
                event_queue.dispatch_pending(&mut state)?;

                match guard.read() {
                    Ok(_) => {}
                    Err(wayland_client::backend::WaylandError::Io(e))
                        if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => return Err(e.into()),
                }
                event_queue.dispatch_pending(&mut state)?;
            }
            
            // IPC Events
            if poll_fds[1].revents & libc::POLLIN != 0 {
                ipc_server.handle_connections(&mut state);
            }

            // Config Changes
            if poll_fds[2].revents & libc::POLLIN != 0  {
                if config_watcher.handle()?{
                    let mut context = state.context.borrow_mut();
                    context.pending_apply_conf = true;
                    // asking for a conf reload with immediate manage sequence needed
                    context.ask_manage_dirty()?;
                    drop(context);                  
                }
            }
        }
    }

    // Cleaning after stop
    info!("\n -> Cleaning Socket after Kawa stopped");
    drop(ipc_server);
    drop(config_watcher);
    info!("\n IPC Socket deleted");

    Ok(())
}
