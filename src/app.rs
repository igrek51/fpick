use anyhow::Result;
use ratatui::widgets::ListState;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::fs::File;
use std::sync::mpsc;
use std::thread;

use crate::filesystem::{list_files, FileNode};
use crate::numbers::ClampNumExt;
use crate::tree::{render_tree_nodes, TreeNode};
use crate::tui::Tui;

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub parent_nodes: Vec<FileNode>, // nodes leading to the current directory
    pub child_nodes: Vec<FileNode>,  // nodes in the current directory
    pub child_tree_nodes: Vec<TreeNode>, // nodes of whole filesystem tree
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

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn post_exit(&mut self) {
        if let Some(picked_path) = &self.picked_path {
            println!("{}", picked_path);
        } else {
            self.exit_code = 1;
        }
    }

    pub fn set_dir_cursor(&mut self, cursor: usize) {
        self.dir_cursor = (cursor as i32)
            .clamp_max(self.child_tree_nodes.len() as i32 - 1)
            .clamp_min(0) as usize;
        if self.child_tree_nodes.is_empty() {
            self.file_tree_state.select(None);
        } else {
            self.file_tree_state.select(Some(self.dir_cursor));
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        let new_cursor = (self.dir_cursor as i32 + delta)
            .clamp_max(self.child_tree_nodes.len() as i32 - 1)
            .clamp_min(0) as usize;
        self.set_dir_cursor(new_cursor);
    }

    pub fn get_current_string_path(&self) -> String {
        if self.parent_nodes.is_empty() {
            return ".".to_string();
        }
        self.parent_nodes
            .iter()
            .map(|node| node.name.to_string())
            .collect::<Vec<String>>()
            .join("/")
    }

    pub fn populate_current_child_nodes(&mut self) {
        let path = self.get_current_string_path();

        let mut nodes = list_files(std::path::Path::new(&path)).unwrap_or_default();
        nodes.sort_by(|a, b| {
            if a.file_type == crate::filesystem::FileType::Directory
                && b.file_type != crate::filesystem::FileType::Directory
            {
                std::cmp::Ordering::Less
            } else if a.file_type != crate::filesystem::FileType::Directory
                && b.file_type == crate::filesystem::FileType::Directory
            {
                std::cmp::Ordering::Greater
            } else {
                a.lowercase_name.cmp(&b.lowercase_name)
            }
        });

        self.child_nodes = nodes;
        self.render_tree_nodes();
    }

    pub fn go_up(&mut self) {
        if self.parent_nodes.is_empty() {
            return;
        }
        self.filter_text.clear();
        let parent: FileNode = self.parent_nodes.pop().unwrap();
        self.populate_current_child_nodes();
        let new_cursor = self
            .child_tree_nodes
            .iter()
            .position(|node| node.file_node.name == parent.name);
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

    pub fn get_selected_file_node(&self) -> Option<FileNode> {
        if self.child_tree_nodes.is_empty() || self.dir_cursor >= self.child_tree_nodes.len() {
            return None;
        }
        Some(self.child_tree_nodes[self.dir_cursor].file_node.clone())
    }

    pub fn go_into(&mut self) {
        if self.child_tree_nodes.is_empty() {
            return;
        }
        let selected_node_o: Option<FileNode> = self.get_selected_file_node();
        if selected_node_o.is_none() {
            return;
        }
        let selected_node: FileNode = selected_node_o.unwrap();
        if selected_node.file_type != crate::filesystem::FileType::Directory {
            return;
        }
        self.parent_nodes.push(selected_node);
        self.filter_text.clear();
        self.populate_current_child_nodes();
        self.set_dir_cursor(0);
    }

    pub fn pick_file(&mut self) {
        if self.child_tree_nodes.is_empty() {
            return;
        }
        let selected_node_o: Option<FileNode> = self.get_selected_file_node();
        if selected_node_o.is_none() {
            return;
        }
        let selected_node: FileNode = selected_node_o.unwrap();

        let mut all_nodes = self.parent_nodes.clone();
        all_nodes.push(selected_node);
        let selected_path = all_nodes
            .iter()
            .map(|node| node.name.to_string())
            .collect::<Vec<String>>()
            .join("/");

        self.picked_path = Some(selected_path);
        self.quit();
    }

    pub fn render_tree_nodes(&mut self) {
        self.child_tree_nodes = render_tree_nodes(&self.child_nodes, &self.filter_text);
        self.move_cursor(0); // validate cursor position
    }
}
