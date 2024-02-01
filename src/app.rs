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
    pub picked_path: Option<String>,
    pub exit_code: i32,
}

impl App {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn run(&mut self) -> Result<()> {
        self.pre_init();
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
        self.post_exit();
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

    pub fn pre_init(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        if args.last().map(|s| s == "--version").unwrap_or(false) {
            println!("{}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
    }

    pub fn init(&mut self) {
        self.populate_current_child_nodes();
        self.set_dir_cursor(0);
    }

    pub fn set_dir_cursor(&mut self, cursor: usize) {
        self.dir_cursor = (cursor as i32)
            .clamp_max(self.current_child_nodes.len() as i32 - 1)
            .clamp_min(0) as usize;
        if self.current_child_nodes.is_empty() {
            self.file_tree_state.select(None);
        } else {
            self.file_tree_state.select(Some(self.dir_cursor));
        }
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

    pub fn move_cursor(&mut self, delta: i32) {
        let new_cursor = (self.dir_cursor as i32 + delta)
            .clamp_max(self.current_child_nodes.len() as i32 - 1)
            .clamp_min(0) as usize;
        self.set_dir_cursor(new_cursor);
    }

    pub fn go_up(&mut self) {
        if self.current_nodes.is_empty() {
            return;
        }
        let parent: FileNode = self.current_nodes.pop().unwrap();
        self.populate_current_child_nodes();
        let new_cursor = self
            .current_child_nodes
            .iter()
            .position(|node| node.name == parent.name);
        match new_cursor {
            Some(idx) => {
                self.dir_cursor = idx;
            }
            None => {
                self.dir_cursor = 0;
            }
        }
        self.set_dir_cursor(self.dir_cursor);
    }

    pub fn go_into(&mut self) {
        if self.current_child_nodes.is_empty() {
            return;
        }
        let selected_node: FileNode = self.current_child_nodes[self.dir_cursor].clone();
        if selected_node.file_type != crate::filesystem::FileType::Directory {
            return;
        }
        self.current_nodes.push(selected_node);
        self.populate_current_child_nodes();
        self.set_dir_cursor(0);
    }

    pub fn pick_file(&mut self) {
        if self.current_child_nodes.is_empty() {
            return;
        }
        let selected_node: FileNode = self.current_child_nodes[self.dir_cursor].clone();

        let mut all_nodes = self.current_nodes.clone();
        all_nodes.push(selected_node);

        let selected_path = all_nodes
            .iter()
            .map(|node| node.name.to_string())
            .collect::<Vec<String>>()
            .join("/");

        self.picked_path = Some(selected_path);
        self.quit();
    }

    pub fn post_exit(&mut self) {
        if let Some(picked_path) = &self.picked_path {
            println!("{}", picked_path);
        } else {
            self.exit_code = 1;
        }
    }
}
