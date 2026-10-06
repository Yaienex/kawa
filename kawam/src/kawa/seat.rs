//! Seat management for Kawa Client

use wayland_client::protocol::wl_seat::WlSeat;

// Seat Identifier
pub type SeatId = usize;

pub struct Seat {
    pub wl_seat_name: u32,
    pub wl_seat: Option<WlSeat>,
    // to know if the seat has been removed and must be destroy
    pub removed: bool,
}
