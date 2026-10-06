use std::{ io, os::fd::{AsRawFd, RawFd},};

use inotify::{EventMask, Inotify, WatchMask};
use tracing::{info, warn};

use crate::config::config_path;


pub struct ConfigWatcher {
   pub inotify: Inotify,
}
 
impl ConfigWatcher {
    pub fn new() -> io::Result<Self> {
        let notify = Inotify::init()?;
        notify.watches().add(config_path(), WatchMask::CLOSE_WRITE | WatchMask::MODIFY)?;
        
        Ok(Self {
            inotify: notify, 
        })
    }
 
    /// get the file descriptor to poll
    pub fn as_raw_fd(&self) -> RawFd {
        self.inotify.as_raw_fd()
    }

    /// handle events and choose to toggle apply_pending_config
    pub fn handle(&mut self) -> io::Result<bool> {
        let mut buffer = [0u8;4096];
        let events = self.inotify
                    .read_events_blocking(&mut buffer)
                    .expect("Failed to read inotify events");

        for event in events{
            if event.mask.contains(EventMask::CLOSE_WRITE){
                info!("close write event");
                return Ok(true);
            }
        }
        Ok(false)
    }
}
