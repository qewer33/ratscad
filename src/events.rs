use std::time::Duration;

use crossterm::event::{self, Event};

pub enum MeshMsg {
    Started,
    Ready { source: String, bytes: Vec<u8> },
    Failed(String),
}

pub fn poll_input(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
