//! Seat management for Kawa Client

use wayland_client::protocol::wl_seat::WlSeat;
use super::window::Window;
use crate::protocol::river::RiverSeatV1;

// Seat Identifier
pub type SeatId = usize;

pub struct Seat {
    pub id: SeatId,
    pub rwm_seat: Option<RiverSeatV1>,
    pub wl_seat: Option<WlSeat>,

    pub wl_seat_name: u32,
    // to know if the seat has been removed and must be destroy
    pub removed: bool,
}

impl Seat{
    pub fn new(id: SeatId) -> Self{
        Self { 
            id: id, 
            rwm_seat: None,
            wl_seat: None,
            removed: false,
            wl_seat_name: 0 }
    }

    /// Focus a window
    pub fn focus_window(&self, window: &Window) {
        if let (Some(rwm_seat), Some(rwm_window)) = (&self.rwm_seat, &window.rwm_window) {
            rwm_seat.focus_window(rwm_window);
        }
    }
}