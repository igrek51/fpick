use anyhow::Result;
use ratatui::widgets::ListState;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::sync::mpsc;
use std::thread;

use crate::filesystem::FileNode;
use crate::logs::print_logs;
use crate::tree::TreeNode;
use crate::tui::Tui;

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub starting_dir: String,
    pub relative_path: bool,
    pub absolute_path: bool,
    pub print_stderr: bool,
    pub parent_nodes: Vec<FileNode>, // nodes leading to the current directory
    pub starting_dir_nodes: Vec<FileNode>, // nodes leading to the starting directory
    pub child_nodes: Vec<FileNode>,  // nodes in the current directory
    pub child_tree_nodes: Vec<TreeNode>, // nodes of filesystem tree to display
    pub dir_cursor: usize,
    pub filter_text: String,
    pub file_tree_state: ListState,
    pub picked_path: Option<String>,
    pub exit_code: i32,
    pub error_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn run(&mut self) -> Result<()> {
        self.pre_init()?;
        let signal_rx = self.handle_signals();
        self.init_catch();
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
        self.post_exit();
        print_logs();
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
