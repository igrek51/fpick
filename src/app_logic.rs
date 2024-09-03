use anyhow::{anyhow, Context, Result};
use crossterm::tty::IsTty;
use relative_path::{PathExt, RelativePathBuf, RelativeToError};
use std::fs;
use std::io::stdout;
use std::path::{Path, PathBuf};

use crate::action_menu::{execute_shell_operation, rename_file, MenuAction, Operation};
use crate::app::App;
use crate::appdata::WindowFocus;
use crate::errors::contextualized_error;
use crate::filesystem::{
    get_path_file_nodes, get_string_abs_path, list_files, nodes_start_with, trim_end_slash,
    FileNode, FileType,
};
use crate::numbers::ClampNumExt;
use crate::tree::{render_tree_nodes, TreeNode, TreeNodeType};
use crate::tui::Tui;

const HELP_TEXT: &str = "fpick - interactive file picker. 
Navigate with arrow keys and enter. It returns the selected path to standard output.

Usage:
  `fpick [OPTIONS]` to select a file in a current directory and return its path
  `fpick [OPTIONS] <PATH>` to select a file starting from a specified directory

Options:
    --relative, --rel, -r  Always print chosen path as relative to the starting directory
    --absolute, --abs, -a  Always print chosen path as absolute directory
    --version              Print version
    --help, -h             Print usage
";

impl App {
    pub fn pre_init(&mut self) -> Result<()> {
        let mut args: Vec<String> = std::env::args().collect::<Vec<String>>()[1..].to_vec();
        args.reverse();
        while args.len() > 0 {
            let arg = args.pop().unwrap();
            match arg.as_str() {
                "--version" => {
                    println!("{}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                "--help" | "-h" => {
                    print!("{}", HELP_TEXT);
                    std::process::exit(0);
                }
                "--relative" | "--rel" | "-r" => {
                    self.relative_path = true;
                }
                "--absolute" | "--abs" | "-a" => {
                    self.absolute_path = true;
                }
                "--stderr" => {
                    self.print_stderr = true;
                }
                _ => {
                    if !self.starting_dir.is_empty() {
                        return Err(anyhow!(
                            "unrecognized arguments or too many arguments. Use --help for usage"
                        ));
                    }
                    self.starting_dir = trim_end_slash(arg.to_string());
                }
            }
        }
        Ok(())
    }

    pub fn init_catch(&mut self) {
        if let Err(e) = self.init() {
            self.error_message = Some(contextualized_error(&e));
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.parent_file_nodes =
            get_path_file_nodes(&self.starting_dir).context("reading path nodes")?;
        self.starting_dir_nodes = self.parent_file_nodes.clone();
        self.populate_current_child_nodes();
        self.set_dir_cursor(0);
        Ok(())
    }

    pub fn render_tree_nodes(&mut self) {
        self.child_tree_nodes = render_tree_nodes(&self.child_file_nodes, &self.filter_text);
        self.reset_cursor_offset();
        self.move_cursor(0); // validate cursor position
    }

    pub fn post_exit(&mut self) {
        if let Some(picked_path) = &self.picked_path {
            println!("{}", picked_path);
            if !stdout().is_tty() && self.print_stderr {
                eprintln!("{}", picked_path);
            }
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

    pub fn reset_cursor_offset(&mut self) {
        self.file_tree_state = self.file_tree_state.clone().with_offset(0);
    }

    pub fn move_cursor(&mut self, delta: i32) {
        match self.window_focus {
            WindowFocus::Tree => {
                let new_cursor = (self.dir_cursor as i32 + delta)
                    .clamp_max(self.child_tree_nodes.len() as i32 - 1)
                    .clamp_min(0) as usize;
                self.set_dir_cursor(new_cursor);
            }
            WindowFocus::ActionMenu => {
                let new_cursor = (self.action_cursor as i32 + delta)
                    .clamp_max(self.known_menu_actions.len() as i32 - 1)
                    .clamp_min(0) as usize;
                self.action_cursor = new_cursor;
            }
            _ => {}
        }
    }

    pub fn get_current_string_path(&self) -> String {
        get_string_abs_path(&self.parent_file_nodes)
    }

    pub fn populate_current_child_nodes(&mut self) {
        let path = self.get_current_string_path();
        let nodes_result = list_files(std::path::Path::new(&path));
        if nodes_result.is_err() {
            self.error_message = Some(contextualized_error(&nodes_result.unwrap_err()));
            self.child_file_nodes = vec![];
            self.render_tree_nodes();
            return;
        }
        let mut nodes = nodes_result.unwrap();
        nodes.sort_by(|a, b| {
            if a.file_type == FileType::Directory && b.file_type != FileType::Directory {
                std::cmp::Ordering::Less
            } else if a.file_type != FileType::Directory && b.file_type == FileType::Directory {
                std::cmp::Ordering::Greater
            } else {
                a.lowercase_name.cmp(&b.lowercase_name)
            }
        });
        self.child_file_nodes = nodes;
        self.render_tree_nodes();
    }

    pub fn go_up(&mut self) {
        if self.parent_file_nodes.is_empty() {
            return;
        }
        self.filter_text.clear();
        let parent: FileNode = self.parent_file_nodes.pop().unwrap();
        self.populate_current_child_nodes();
        let new_cursor = self
            .child_tree_nodes
            .iter()
            .position(|node| node.name() == parent.name);
        match new_cursor {
            Some(idx) => {
                self.dir_cursor = idx;
            }
            None => {
                self.dir_cursor = 0;
            }
        }
        self.reset_cursor_offset();
        self.set_dir_cursor(self.dir_cursor);
    }

    pub fn go_into(&mut self) {
        let selected_node_o: Option<TreeNode> = self.get_selected_tree_node();
        if selected_node_o.is_none() {
            return;
        }
        match selected_node_o.unwrap().kind {
            TreeNodeType::SelfReference => return,
            TreeNodeType::FileNode(file_node) => {
                if file_node.file_type != FileType::Directory {
                    return;
                }
                self.parent_file_nodes.push(file_node.clone());
                self.filter_text.clear();
                self.populate_current_child_nodes();
                self.reset_cursor_offset();
                self.set_dir_cursor(0);
            }
        }
    }

    pub fn go_to_root(&mut self) {
        self.parent_file_nodes = vec![];
        self.filter_text.clear();
        self.populate_current_child_nodes();
        self.reset_cursor_offset();
        self.set_dir_cursor(0);
    }

    pub fn enter_selected_node(&mut self) {
        let selected_node_o: Option<TreeNode> = self.get_selected_tree_node();
        if selected_node_o.is_none() {
            return;
        }
        match selected_node_o.unwrap().kind {
            TreeNodeType::SelfReference => {
                self.pick_current_dir();
            }
            TreeNodeType::FileNode(file_node) => match file_node.file_type {
                FileType::Directory => self.go_into(),
                _ => self.pick_selected_node(None),
            },
        }
    }

    pub fn pick_current_dir(&mut self) {
        let current_dir_node = TreeNode {
            relevance: 0,
            kind: TreeNodeType::SelfReference,
        };
        self.pick_tree_node(&current_dir_node, None);
    }

    pub fn pick_selected_node(&mut self, relative_mode: Option<bool>) {
        let selected_node_o: Option<TreeNode> = self.get_selected_tree_node();
        if selected_node_o.is_none() {
            return;
        }
        let selected_node: TreeNode = selected_node_o.unwrap();
        self.pick_tree_node(&selected_node, relative_mode);
    }

    pub fn pick_tree_node(&mut self, tree_node: &TreeNode, relative_mode_o: Option<bool>) {
        let (chosen_path, chosen_nodes) = self.build_tree_node_path(tree_node);
        let relative_mode: bool = match relative_mode_o {
            Some(b) => b,
            None => self.determine_relative_mode(&chosen_nodes),
        };
        self.picked_path = match relative_mode {
            true => self.make_relative_path(&chosen_path),
            false => Some(chosen_path),
        };
        if !self.picked_path.is_none() {
            self.quit();
        }
    }

    pub fn get_selected_tree_node(&self) -> Option<TreeNode> {
        if self.child_tree_nodes.is_empty() || self.dir_cursor >= self.child_tree_nodes.len() {
            return None;
        }
        let selected_node = &self.child_tree_nodes[self.dir_cursor];
        Some(selected_node.clone())
    }

    pub fn build_tree_node_path(&self, tree_node: &TreeNode) -> (String, Vec<FileNode>) {
        match &tree_node.kind {
            TreeNodeType::SelfReference => {
                return (
                    get_string_abs_path(&self.parent_file_nodes),
                    self.parent_file_nodes.clone(),
                );
            }
            TreeNodeType::FileNode(file_node) => {
                let mut chosen_nodes: Vec<FileNode> = self.parent_file_nodes.clone();
                chosen_nodes.push(file_node.clone());
                return (get_string_abs_path(&chosen_nodes), chosen_nodes);
            }
        }
    }

    pub fn get_selected_abs_path(&mut self) -> Option<String> {
        let selected_node_o: Option<TreeNode> = self.get_selected_tree_node();
        if selected_node_o.is_none() {
            return None;
        }
        let (chosen_path, _) = self.build_tree_node_path(&selected_node_o.unwrap());
        return Some(chosen_path);
    }

    pub fn determine_relative_mode(&self, chosen_nodes: &Vec<FileNode>) -> bool {
        if self.absolute_path {
            return false;
        }
        if self.relative_path {
            return true;
        }
        nodes_start_with(chosen_nodes, &self.starting_dir_nodes)
    }

    pub fn make_relative_path(&mut self, chosen_path: &String) -> Option<String> {
        let selected_path: &Path = Path::new(&chosen_path);
        let starting_path: &Path = match self.starting_dir.is_empty() {
            true => Path::new("."),
            false => Path::new(&self.starting_dir),
        };
        let starting_path_abs: PathBuf = fs::canonicalize(&starting_path).unwrap();

        let relative_path_r: Result<RelativePathBuf, RelativeToError> =
            selected_path.relative_to(starting_path_abs);
        let relative_path: String = match relative_path_r {
            Err(_) => {
                self.error_message = Some(format!(
                    "Selected path is not relative to the starting directory"
                ));
                return None;
            }
            Ok(res) => res.to_string(),
        };
        match relative_path.is_empty() {
            true => Some(String::from(".")),
            false => Some(relative_path.to_string()),
        }
    }

    pub fn type_search_text(&mut self, c: char) {
        self.filter_text.push(c);
        self.render_tree_nodes();
        self.set_dir_cursor(0);
    }

    pub fn backspace_search_text(&mut self) {
        self.filter_text.pop();
        self.render_tree_nodes();
    }

    pub fn clear_search_text(&mut self) {
        self.filter_text.clear();
        self.render_tree_nodes();
    }

    pub fn has_error(&self) -> bool {
        self.error_message.is_some()
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn open_action_dialog(&mut self) {
        if self.child_tree_nodes.is_empty() {
            return;
        }
        self.window_focus = WindowFocus::ActionMenu;
        self.action_cursor = 0;
    }

    pub fn close_action_dialog(&mut self) {
        self.window_focus = WindowFocus::Tree;
    }

    pub fn execute_dialog_action(&mut self, _: &mut Tui) {
        let path = self.get_selected_abs_path();
        if path.is_none() {
            return;
        }

        let action: &MenuAction = &self.known_menu_actions[self.action_cursor];

        match action.operation {
            Operation::ShellCommand { template } => {
                let res = execute_shell_operation(&path.unwrap(), template);
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                }
                self.window_focus = WindowFocus::Tree;
            }
            Operation::PickAbsolutePath => {
                self.pick_selected_node(Some(false));
            }
            Operation::PickRelativePath => {
                self.pick_selected_node(Some(true));
            }
            Operation::Rename => {
                let filename = path.unwrap().split('/').last().unwrap().to_string();
                self.window_focus = WindowFocus::ActionMenuStep2;
                self.action_menu_operation = Some(Operation::Rename);
                self.action_menu_title = "New name".to_string();
                self.action_menu_buffer = filename;
            }
        }
        self.populate_current_child_nodes();
    }
    pub fn execute_dialog_action_step2(&mut self, _: &mut Tui) {
        let path = self.get_selected_abs_path();
        if path.is_none() {
            return;
        }

        match self.action_menu_operation {
            Some(Operation::Rename) => {
                let res = rename_file(&path.unwrap(), &self.action_menu_buffer);
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                }
                self.window_focus = WindowFocus::Tree;
            }
            _ => {}
        }
        self.populate_current_child_nodes();
    }
}
