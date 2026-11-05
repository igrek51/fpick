use std::time::Duration;

use anyhow::Result;
use crossterm::event::KeyEvent;

#[cfg(unix)]
use crate::tty_input::TtyInput;

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
/// On Unix keys are read one at a time straight from the tty, so a key pressed
/// right after the quit key stays in the terminal input queue for the shell.
/// Other platforms fall back to crossterm's reader.
#[derive(Debug)]
pub struct EventHandler {
    tick_rate: Duration,
    #[cfg(unix)]
    input: TtyInput,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Result<Self> {
        Ok(Self {
            tick_rate: Duration::from_millis(tick_rate),
            #[cfg(unix)]
            input: TtyInput::new()?,
        })
    }

    /// Wait up to the tick rate for the next event. Returns `None` on timeout.
    pub fn next(&self) -> Result<Option<Event>> {
        self.poll(self.tick_rate)
    }

    /// Take an event that is already available, without waiting.
    pub fn try_next(&self) -> Result<Option<Event>> {
        self.poll(Duration::ZERO)
    }

    #[cfg(unix)]
    fn poll(&self, timeout: Duration) -> Result<Option<Event>> {
        self.input.next_event(timeout)
    }

    #[cfg(not(unix))]
    fn poll(&self, timeout: Duration) -> Result<Option<Event>> {
        use crossterm::event::{self, Event as CrosstermEvent, KeyEventKind};

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
