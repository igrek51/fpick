use anyhow::Result;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::sync::mpsc;
use std::thread;

use crate::tui::Tui;

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub dir_cursor: usize,
    pub filter_text: String,
}

impl App {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn run(&mut self) -> Result<()> {
        let signal_rx = self.handle_signals();
        let mut tui = Tui::new();
        tui.enter()?;

        while !self.should_quit {
            tui.draw(self)?;
            tui.handle_events(self)?;

            signal_rx.try_recv().ok().map(|_| {
                self.quit();
            });
        }

        tui.exit()?;
        Ok(())
    }

    pub fn handle_signals(&mut self) -> mpsc::Receiver<i32> {
        let (tx, rx) = mpsc::channel();
        let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
        thread::spawn(move || {
            for sig in signals.forever() {
                println!("Received signal {:?}", sig);
                tx.send(sig).unwrap();
            }
        });
        return rx;
    }

    pub fn tick(&mut self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
