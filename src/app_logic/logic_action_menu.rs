use std::str::Chars;

use crate::action_menu::{
    copy_path_to_clipboard, create_directory, create_file, delete_tree_node,
    execute_interactive_shell_operation, execute_shell_operation, get_file_details, rename_file,
    MenuAction, Operation,
};
use crate::app::App;
use crate::appdata::WindowFocus;
use crate::tree::TreeNode;
use crate::tui::Tui;

impl App {
    pub fn open_action_dialog(&mut self) {
        if self.child_tree_nodes.is_empty() {
            return;
        }
        self.window_focus = WindowFocus::ActionMenu;
        self.action_menu_cursor_y = 0;
    }

    pub fn close_action_dialog(&mut self) {
        self.window_focus = WindowFocus::Tree;
    }

    pub fn execute_dialog_action(&mut self, tui: &mut Tui) {
        let abs_path: String = match self.get_selected_abs_path() {
            Some(abs_path) => abs_path,
            None => return,
        };
        let tree_node: TreeNode = match self.get_selected_tree_node() {
            Some(tree_node) => tree_node,
            None => return,
        };
        let relative_path: Option<String> = self.make_relative_path(&abs_path);
        let is_directory = App::is_tree_node_directory(&tree_node);

        let action: &MenuAction = &self.known_menu_actions[self.action_menu_cursor_y];
        match action.operation {
            Operation::ShellCommand { template } => {
                self.window_focus = WindowFocus::Tree;
                self.error_message = Some("executing".to_string());
                let res = execute_shell_operation(&abs_path, template);
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                }
            }
            Operation::InteractiveShellCommand { template } => {
                let res = execute_interactive_shell_operation(&abs_path, template, tui);
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
                let filename = abs_path.split('/').last().unwrap().to_string();
                self.window_focus = WindowFocus::ActionMenuStep2;
                self.action_menu_operation = Some(action.operation.clone());
                self.action_menu_title = format!("New name for {}", filename);
                self.action_menu_buffer = filename;
                self.action_menu_cursor_x = self.action_menu_buffer.chars().count();
            }
            Operation::CreateFile => {
                let current_dir_path: String = self.get_current_dir_abs_path();
                self.window_focus = WindowFocus::ActionMenuStep2;
                self.action_menu_operation = Some(action.operation.clone());
                self.action_menu_title = format!("New file at {}", current_dir_path);
                self.action_menu_buffer = "".to_string();
                self.action_menu_cursor_x = self.action_menu_buffer.chars().count();
            }
            Operation::CreateDir => {
                let current_dir_path: String = self.get_current_dir_abs_path();
                self.window_focus = WindowFocus::ActionMenuStep2;
                self.action_menu_operation = Some(action.operation.clone());
                self.action_menu_title = format!("New directory at {}", current_dir_path);
                self.action_menu_buffer = "".to_string();
                self.action_menu_cursor_x = self.action_menu_buffer.chars().count();
            }
            Operation::Delete => {
                let res = delete_tree_node(&tree_node, &abs_path);
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                }
                self.window_focus = WindowFocus::Tree;
            }
            Operation::CopyToClipboard { is_relative_path } => {
                self.window_focus = WindowFocus::Tree;
                let res = match is_relative_path {
                    true => copy_path_to_clipboard(&relative_path.unwrap()),
                    false => copy_path_to_clipboard(&abs_path),
                };
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                }
            }
            Operation::FileDetails => {
                self.window_focus = WindowFocus::Tree;
                let res = get_file_details(&abs_path, is_directory);
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                } else {
                    self.info_message = Some(res.unwrap());
                }
            }
        }
        self.populate_current_child_nodes();
    }

    pub fn execute_dialog_action_step2(&mut self, _: &mut Tui) {
        let path = self.get_selected_abs_path();
        if path.is_none() {
            return;
        }
        if self.action_menu_buffer.is_empty() {
            self.error_message = Some("No value given".to_string());
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
            Some(Operation::CreateFile) => {
                let mut current_dir_path: String = self.get_current_dir_abs_path();
                current_dir_path.push('/');
                current_dir_path.push_str(&self.action_menu_buffer);
                let res = create_file(&current_dir_path);
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                }
                self.window_focus = WindowFocus::Tree;
            }
            Some(Operation::CreateDir) => {
                let mut current_dir_path: String = self.get_current_dir_abs_path();
                current_dir_path.push('/');
                current_dir_path.push_str(&self.action_menu_buffer);
                let res = create_directory(&current_dir_path);
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                }
                self.window_focus = WindowFocus::Tree;
            }
            _ => {}
        }
        self.populate_current_child_nodes();
    }

    pub fn action_menu_input_append(&mut self, c: char) {
        let chars: Chars<'_> = self.action_menu_buffer.chars();
        let cx = self.action_menu_cursor_x;
        let before: String = chars.clone().take(cx).collect::<String>();
        let after: String = chars.skip(cx).collect::<String>();
        self.action_menu_buffer = before + &c.to_string() + &after;
        self.action_menu_input_right();
    }

    pub fn action_menu_input_clear_backwards(&mut self) {
        let chars: Chars<'_> = self.action_menu_buffer.chars();
        let cx = self.action_menu_cursor_x;
        let after: String = chars.skip(cx).collect::<String>();
        self.action_menu_buffer = after;
        self.action_menu_cursor_x = 0;
    }

    pub fn action_menu_input_backspace(&mut self) {
        let chars: Chars<'_> = self.action_menu_buffer.chars();
        let cx = self.action_menu_cursor_x;
        let mut before: String = chars.clone().take(cx).collect::<String>();
        let after: String = chars.skip(cx).collect::<String>();
        if !before.is_empty() {
            before.pop();
            self.action_menu_buffer = before + &after;
            self.action_menu_input_left();
        }
    }

    pub fn action_menu_input_delete(&mut self) {
        let chars: Chars<'_> = self.action_menu_buffer.chars();
        let cx = self.action_menu_cursor_x;
        let before: String = chars.clone().take(cx).collect::<String>();
        let mut after: String = chars.skip(cx).collect::<String>();
        if !after.is_empty() {
            after.remove(0);
            self.action_menu_buffer = before + &after;
        }
    }

    pub fn action_menu_input_left(&mut self) {
        if self.action_menu_cursor_x > 0 {
            self.action_menu_cursor_x -= 1;
        }
    }

    pub fn action_menu_input_right(&mut self) {
        self.action_menu_cursor_x += 1;
        let length = self.action_menu_buffer.chars().count();
        if self.action_menu_cursor_x > length {
            self.action_menu_cursor_x = length;
        }
    }

    pub fn action_menu_input_home(&mut self) {
        self.action_menu_cursor_x = 0;
    }

    pub fn action_menu_input_end(&mut self) {
        self.action_menu_cursor_x = self.action_menu_buffer.chars().count();
    }
}
