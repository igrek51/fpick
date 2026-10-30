use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Key press.
    Key(KeyEvent),
    /// Terminal resize.
    Resize,
}

/// Terminal event handler.
///
/// Events are polled on demand by the thread that owns the UI. Nothing reads the
/// terminal in the background, so keys pressed while the app is busy stay in the
/// terminal input queue instead of being consumed and thrown away.
#[derive(Debug)]
pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate),
        }
    }

    /// Wait up to the tick rate for the next event. Returns `None` on timeout.
    pub fn next(&self) -> Result<Option<Event>> {
        self.poll(self.tick_rate)
    }

    /// Take an event that is already available, without waiting.
    pub fn try_next(&self) -> Result<Option<Event>> {
        self.poll(Duration::ZERO)
    }

    fn poll(&self, timeout: Duration) -> Result<Option<Event>> {
        if !event::poll(timeout)? {
            return Ok(None);
        }
        match event::read()? {
            CrosstermEvent::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                Ok(Some(Event::Key(key_event)))
            }
            // Ignore key releases, repeats and other non-key events.
            CrosstermEvent::Key(_) => Ok(None),
            CrosstermEvent::Resize(_, _) => Ok(Some(Event::Resize)),
            _ => Ok(None),
        }
    }
}
