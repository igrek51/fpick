use anyhow::Result;
use ratatui::widgets::ListState;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::sync::mpsc;
use std::thread;

use crate::action_menu::{generate_known_actions, MenuAction, Operation};
use crate::appdata::WindowFocus;
use crate::background::BackgroundEvent;
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
    pub parent_file_nodes: Vec<FileNode>, // nodes leading to the current directory
    pub starting_dir_nodes: Vec<FileNode>, // nodes leading to the starting directory
    pub child_file_nodes: Vec<FileNode>,  // nodes in the current directory
    pub child_tree_nodes: Vec<TreeNode>,  // nodes of filesystem tree to display
    pub dir_cursor: usize,
    pub filter_text: String,
    pub file_tree_state: ListState,
    pub picked_path: Option<String>,
    pub exit_code: i32,
    pub error_message: Option<String>,
    pub info_message: Option<String>,
    pub info_message_scroll: usize,
    pub window_focus: WindowFocus,
    pub known_menu_actions: Vec<MenuAction>,
    pub action_menu_cursor_y: usize,
    pub action_menu_cursor_x: usize,
    pub action_menu_operation: Option<Operation>,
    pub action_menu_title: String,
    pub action_menu_buffer: String,
    pub background_event_channel: BackgroundEventChannel,
}

#[derive(Debug)]
pub struct BackgroundEventChannel {
    pub tx: mpsc::Sender<BackgroundEvent>,
    pub rx: mpsc::Receiver<BackgroundEvent>,
}

impl Default for BackgroundEventChannel {
    fn default() -> Self {
        let (background_events_tx, background_events_rx) = mpsc::channel();
        Self {
            tx: background_events_tx,
            rx: background_events_rx,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            known_menu_actions: generate_known_actions(),
            ..Default::default()
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.pre_init()?;
        let signal_rx = self.handle_signals();
        self.init_catch();
        let mut tui: Tui = Tui::new();
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

    pub fn tick(&mut self) {
        self.check_background_events();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
