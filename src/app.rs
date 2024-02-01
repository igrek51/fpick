use anyhow::Result;
use ratatui::widgets::ListState;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::sync::mpsc;
use std::thread;

use crate::filesystem::{list_files, FileNode};
use crate::numbers::ClampNumExt;
use crate::tui::Tui;

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub current_nodes: Vec<FileNode>,
    pub current_child_nodes: Vec<FileNode>,
    pub dir_cursor: usize,
    pub filter_text: String,
    pub file_tree_state: ListState,
}

impl App {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn run(&mut self) -> Result<()> {
        let signal_rx = self.handle_signals();
        self.init();
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

    pub fn init(&mut self) {
        self.populate_current_child_nodes();
        self.dir_cursor = 0;
        if !self.current_child_nodes.is_empty() {
            self.file_tree_state.select(Some(self.dir_cursor));
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        self.dir_cursor = (self.dir_cursor as i32 + delta)
            .clamp_min(0)
            .clamp_max(self.current_child_nodes.len() as i32 - 1)
            as usize;
        self.file_tree_state.select(Some(self.dir_cursor));
    }

    pub fn get_current_string_path(&self) -> String {
        if self.current_nodes.is_empty() {
            return ".".to_string();
        }
        self.current_nodes
            .iter()
            .map(|node| node.name.to_string())
            .collect::<Vec<String>>()
            .join("/")
    }

    pub fn populate_current_child_nodes(&mut self) {
        let path = self.get_current_string_path();

        self.current_child_nodes = list_files(std::path::Path::new(&path)).unwrap_or_default();
    }
}
