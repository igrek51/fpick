use std::time::{Duration, Instant};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Terminal resize.
    Resize,
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    suspended_store: Arc<Mutex<bool>>,
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let suspended_store = Arc::new(Mutex::new(false));
        Self {
            sender,
            receiver,
            suspended_store,
            tick_rate,
        }
    }

    pub fn listen(self) -> Self {
        let sender = self.sender.clone();
        let suspended_store: Arc<Mutex<bool>> = self.suspended_store.clone();
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = self
                    .tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(self.tick_rate);

                if Self::is_suspended(&suspended_store) {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                if event::poll(timeout).expect("unable to poll for event") {
                    match event::read().expect("unable to read event") {
                        CrosstermEvent::Key(e) => {
                            if !Self::is_suspended(&suspended_store.clone()) {
                                if e.kind == event::KeyEventKind::Press {
                                    sender
                                        .send(Event::Key(e))
                                        .expect("failed to send key event");
                                }
                            }
                        }
                        CrosstermEvent::Resize(_, _) => {
                            sender
                                .send(Event::Resize)
                                .expect("failed to send resize event");
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= self.tick_rate {
                    sender.send(Event::Tick).expect("failed to send tick event");
                    last_tick = Instant::now();
                }
            }
        });
        self
    }

    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    pub fn suspend(&self) {
        *self.suspended_store.lock().unwrap() = true;
    }

    pub fn resume(&self) {
        Self::clear_queued_events();
        *self.suspended_store.lock().unwrap() = false;
    }

    pub fn clear_queued_events() {
        while Self::is_event_available() {
            event::read().unwrap();
        }
    }

    pub fn is_event_available() -> bool {
        event::poll(Duration::from_millis(100)).unwrap()
    }

    pub fn is_suspended(suspended_store: &Arc<Mutex<bool>>) -> bool {
        *suspended_store.clone().lock().unwrap()
    }
}
