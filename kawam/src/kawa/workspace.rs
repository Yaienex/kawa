use crate::kawa::window::WindowId;

pub type WorkspaceId = usize;


pub struct Workspace {
    pub id: WorkspaceId,
    pub windows: Vec<WindowId>,
    pub focused: Option<WindowId>,
}