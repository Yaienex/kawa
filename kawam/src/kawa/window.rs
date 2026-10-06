//! Window Object manage by Kawa Client
//! So we don't manipulate raw Wayland data


// Window Identifier
pub type WindowId = usize;

pub struct Window {
    pub id: WindowId,
}
