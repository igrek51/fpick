use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
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
        let suspended_store = self.suspended_store.clone();
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = self
                    .tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(self.tick_rate);

                let suspended = *suspended_store.lock().unwrap();
                if suspended {
                    thread::sleep(Duration::from_millis(100));
                } else {
                    if event::poll(timeout).expect("unable to poll for event") {
                        match event::read().expect("unable to read event") {
                            CrosstermEvent::Key(e) => {
                                let suspended = *suspended_store.lock().unwrap();
                                if !suspended && e.kind == event::KeyEventKind::Press {
                                    sender.send(Event::Key(e))
                                } else {
                                    Ok(()) // ignore KeyEventKind::Release on windows
                                }
                            }
                            CrosstermEvent::Resize(_, _) => sender.send(Event::Resize),
                            _ => unimplemented!(),
                        }
                        .expect("failed to send terminal event")
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
        *self.suspended_store.lock().unwrap() = false;
    }
}
