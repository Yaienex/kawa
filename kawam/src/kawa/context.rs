use crate::{config::{Config, lua::LuaRuntime}};
use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use super::{
    output::{Output, OutputId},
    seat::{Seat, SeatId},
    window::{Window, WindowId},
};
use crate::protocol::river::{
    RiverInputManagerV1, RiverLayerShellV1, RiverLibinputConfigV1, RiverWindowManagerV1,
    RiverXkbBindingsV1,
};

pub struct Context {
    // River / Wayland Globals
    pub rwm: Option<RiverWindowManagerV1>,
    pub rwm_xkb_bindings: Option<RiverXkbBindingsV1>,
    pub rwm_layer_shell: Option<RiverLayerShellV1>,
    pub rwm_input_manager: Option<RiverInputManagerV1>,
    pub rwm_libinput_config: Option<RiverLibinputConfigV1>,

    // Managed Objects
    pub outputs: HashMap<OutputId, Rc<RefCell<Output>>>,
    pub windows: HashMap<WindowId, Rc<RefCell<Window>>>,
    pub seats: HashMap<SeatId, Rc<RefCell<Seat>>>,

    // Config
    pub config: Config,
    pub lua: LuaRuntime,
    pub pending_apply_conf: bool,

    pub anim_start_time: Option<Duration>,
    pub request_stop: bool,
}


   
impl Context {
    pub fn new(lua: LuaRuntime, config: Config ) -> anyhow::Result<Self>  {
        let  ctx = Self {
            // River / Wayland Globals
            rwm: None,
            rwm_xkb_bindings: None,
            rwm_layer_shell: None,
            rwm_input_manager: None,
            rwm_libinput_config: None,
            //Managed Objects
            outputs: HashMap::new(),
            windows: HashMap::new(),
            seats: HashMap::new(),

            //Config
            config: config,
            lua: lua,
            pending_apply_conf: false,

            // Animation
            anim_start_time: None,
            request_stop: false,
        };

        Ok(ctx)
        
    }

    /// Ask for a manage sequence 
    pub fn ask_manage_dirty(&self) -> Result<(),String>{
        if let Some(ref rwm) = self.rwm {
            rwm.manage_dirty();
        } else {
           return Err("No River WM has been found inside context".into());
        }
        Ok(())
    }
    /// Config reload
    pub fn reload_config(&mut self) {        
        // TO REDO bcs new and load config execute 
        // Must regenerate a new LuaRunTime instance
        // Or just give up the binds, etc and recreate the table ?
        let continue_config_swap: bool = match self.lua.drop_globals(){
            Ok(continue_config_swap) => continue_config_swap,
            Err(_)=> false
        };

        if continue_config_swap{
            tracing::info!("Continue !!");
        } else {
            // The Lua Globals already have been restored and du to the error
            // the config in the context stay the same
            tracing::info!("Bro stop");
        }
        
    }

    /// Handle manage_start event - process pending window events and arrange windows
    pub fn handle_manage_start(&mut self) {
        // Start by re-applying every conf change if needed
        // We are at the start of the manage sequence so all redefinitions operations
        // are valid.
        if self.pending_apply_conf {
            self.pending_apply_conf = false;
            self.reload_config();
        }

    }

    /// Finish manage sequence
    pub fn finish_manage(&self) {
        if let Some(ref rwm) = self.rwm {
            rwm.manage_finish();
        }
    }

    /// Finish Render sequence
    pub fn finish_render(&self) {
        if let Some(ref rwm) = self.rwm {
            rwm.render_finish();
        }
    }
}
