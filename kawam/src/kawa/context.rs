use wayland_protocols::ext::workspace;

use crate::{config::{Config, lua::LuaRuntime}, kawa::workspace::{Workspace, WorkspaceId}, protocol::{river::{RiverOutputV1, RiverSeatV1, RiverWindowV1}, river_wm::river_window_v1::Edges}};
use std::{cell::{Ref, RefCell}, collections::HashMap, rc::Rc, time::Duration};

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
    pub workspaces: HashMap<WorkspaceId, Rc<RefCell<Workspace>>>,
    pub seats: HashMap<SeatId, Rc<RefCell<Seat>>>,
    
    pub current_output: Option<OutputId>,
    pub current_workspace: WorkspaceId,
    // Config
    pub config: Config,
    pub lua: LuaRuntime,
    pub pending_apply_conf: bool,

    // incremental value for next id
    pub next_window_id : usize,
    pub next_output_id : usize,
    pub next_seat_id : usize,

    pub anim_start_time: Option<Duration>,
    pub request_stop: bool,
}


   
impl Context {
    pub fn new(lua: LuaRuntime, config: Config ) -> anyhow::Result<Self>  {
        let wk = Rc::new(RefCell::new(Workspace::new(1)));
        let mut wk_hash = HashMap::new();
        wk_hash.insert(1, wk);
        let  ctx = Self {
            // River / Wayland Globals
            rwm: None,
            rwm_xkb_bindings: None,
            rwm_layer_shell: None,
            rwm_input_manager: None,
            rwm_libinput_config: None,
            //Managed Objects
            outputs: HashMap::new(),
            // Temporary method but it might need to be done somewhere else
            // But the first workspace has to be set up before any window pop
            workspaces: wk_hash,
            seats: HashMap::new(),

            //
            current_output: None,
            current_workspace: 1, // Always start at the first one

            //Config
            config: config,
            lua: lua,
            pending_apply_conf: false,
            next_window_id: 1,
            next_output_id : 1,
            next_seat_id : 1,

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

        for (_workspace_id, workspace) in  &self.workspaces{
            let mut ws = workspace.borrow_mut();
            let output = self.outputs.get(&self.current_output.unwrap().clone()).unwrap();
            ws.propose_dimensions_to_windows(output.borrow().width, output.borrow().height);

            drop(ws);
        }

    }

    pub fn handle_render_start(&mut self){
        //the first workspace is always present 
        let workspace = self.workspaces.get(&self.current_workspace).unwrap();
        let ws = workspace.borrow();
        // TODO 
        let windows = ws.windows.clone();
        for (_win_id,window) in windows {
            let  w = window.borrow_mut();
            let edges = Edges::all();
            w.set_borders(edges, 0, 0, 0, 0, 0);
            drop(w);

            if let Some(focused_id) = ws.focused_window {
                if let Some(root) = ws.windows.get(&focused_id) {
                    let root = root.borrow();
                    root.place_top();
                }
            }
        }
    }

    /// Creation Functions
    /// Create a Kawa Window and return it for further config
    pub fn create_window(&mut self,  rwm_window: RiverWindowV1) -> Rc<RefCell<Window>> {
        let id = self.next_window_id;
        self.next_window_id += 1;

        // Creating our object 
        let mut window = Window::new(id);
        window.rwm_window = Some(rwm_window);

        let window = Rc::new(RefCell::new(window));
      

        if let Some(ws) = self.workspaces.get(&self.current_workspace){
            let mut workspace = ws.borrow_mut();
            workspace.set_focused_window(id);
            workspace.add_window(id,window.clone());
        };

      

        window
    }

    pub fn create_output(&mut self, rwm_output: RiverOutputV1) -> Rc<RefCell<Output>>{
        let id = self.next_output_id;
        self.next_output_id += 1;
         
        let mut output = Output::new(id);
        output.rwm_output = Some(rwm_output);

        let output = Rc::new(RefCell::new(output));
        self.outputs.insert(id, output.clone());

        // Set as current output if first
        if self.current_output.is_none() {
            self.current_output = Some(id);
        }

        output
    }

    pub fn create_seat(&mut self, rwm_seat: RiverSeatV1) -> Rc<RefCell<Seat>> {
        let id = self.next_seat_id;
        self.next_seat_id += 1;
         
        let mut seat = Seat::new(id);
        seat.rwm_seat = Some(rwm_seat);

        let seat = Rc::new(RefCell::new(seat));

        seat
    }

    /// Creation Functions end


    //// Finish Sequence
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
