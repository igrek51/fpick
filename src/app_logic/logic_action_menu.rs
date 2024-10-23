use std::str::Chars;

use crate::action_menu::{
    copy_path_to_clipboard, create_directory, create_file, delete_tree_node,
    execute_interactive_shell_operation, execute_shell_operation, get_file_details,
    read_file_content, rename_file, run_custom_command, MenuAction, Operation,
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
        let current_dir_path: String = self.get_current_dir_abs_path();

        let action: &MenuAction = &self.known_menu_actions[self.action_menu_cursor_y];
        match action.operation {
            Operation::ShellCommand { template } => {
                self.window_focus = WindowFocus::Tree;
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
                self.window_focus = WindowFocus::ActionMenuStep2;
                self.action_menu_operation = Some(action.operation.clone());
                self.action_menu_title = format!("New file at {}", current_dir_path);
                self.action_menu_buffer = "".to_string();
                self.action_menu_cursor_x = self.action_menu_buffer.chars().count();
            }
            Operation::CreateDir => {
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
                    self.show_info(res.unwrap());
                }
            }
            Operation::CustomCommand => {
                self.window_focus = WindowFocus::ActionMenuStep2;
                self.action_menu_operation = Some(action.operation.clone());
                self.action_menu_title = format!("Run command at {}", current_dir_path);
                self.action_menu_buffer = format!("\"{}\"", abs_path);
                self.action_menu_cursor_x = self.action_menu_buffer.chars().count();
            }
            Operation::ViewContent => {
                self.window_focus = WindowFocus::Tree;
                if !is_directory {
                    let res = read_file_content(&abs_path);
                    if res.is_err() {
                        self.error_message = Some(res.err().unwrap().to_string());
                    } else {
                        self.show_info(res.unwrap());
                    }
                }
            }
        }
        self.populate_current_child_nodes();
    }

    pub fn execute_dialog_action_step2(&mut self, _: &mut Tui) {
        let abs_path: String = match self.get_selected_abs_path() {
            Some(abs_path) => abs_path,
            None => return,
        };
        if self.action_menu_buffer.is_empty() {
            self.error_message = Some("No value given".to_string());
            return;
        }
        let current_dir_path: String = self.get_current_dir_abs_path();

        match self.action_menu_operation {
            Some(Operation::Rename) => {
                let res = rename_file(&abs_path, &self.action_menu_buffer);
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                }
            }
            Some(Operation::CreateFile) => {
                let full_path = format!("{}/{}", current_dir_path, &self.action_menu_buffer);
                let res = create_file(&full_path);
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                }
            }
            Some(Operation::CreateDir) => {
                let full_path = format!("{}/{}", current_dir_path, &self.action_menu_buffer);
                let res = create_directory(&full_path);
                if res.is_err() {
                    self.error_message = Some(res.err().unwrap().to_string());
                }
            }
            Some(Operation::CustomCommand) => {
                let res = run_custom_command(current_dir_path, &self.action_menu_buffer);
                match res {
                    Ok(output) => self.show_info(output),
                    Err(err) => self.error_message = Some(err.to_string()),
                }
            }
            _ => {}
        }
        self.window_focus = WindowFocus::Tree;
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

    pub fn action_menu_input_clear_forward(&mut self) {
        let chars: Chars<'_> = self.action_menu_buffer.chars();
        let cx = self.action_menu_cursor_x;
        let before: String = chars.clone().take(cx).collect::<String>();
        self.action_menu_buffer = before;
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

    pub fn action_menu_input_backspace_word(&mut self) {
        let chars: Chars<'_> = self.action_menu_buffer.chars();
        let cx = self.action_menu_cursor_x;
        let before: String = chars.clone().take(cx).collect::<String>();
        let after: String = chars.skip(cx).collect::<String>();
        let mut before_words = before.split(' ').collect::<Vec<&str>>();
        if !before_words.is_empty() {
            if before_words.last().unwrap().is_empty() {
                before_words.pop();
            } else {
                *before_words.last_mut().unwrap() = "";
            }
            let before: String = before_words.join(" ");
            self.action_menu_cursor_x = before.chars().count();
            self.action_menu_buffer = before + &after;
        }
    }

    pub fn action_menu_input_delete_word(&mut self) {
        let chars: Chars<'_> = self.action_menu_buffer.chars();
        let cx = self.action_menu_cursor_x;
        let before: String = chars.clone().take(cx).collect::<String>();
        let after: String = chars.skip(cx).collect::<String>();
        let mut after_words = after.split(' ').collect::<Vec<&str>>();
        if !after_words.is_empty() {
            if after_words.first().unwrap().is_empty() {
                after_words.remove(0);
            } else {
                *after_words.first_mut().unwrap() = "";
            }
            let after: String = after_words.join(" ");
            self.action_menu_buffer = before + &after;
        }
    }

    pub fn action_menu_input_left_word(&mut self) {
        let chars: Chars<'_> = self.action_menu_buffer.chars();
        let cx = self.action_menu_cursor_x;
        let before: String = chars.clone().take(cx).collect::<String>();
        let mut before_words = before.split(' ').collect::<Vec<&str>>();
        if !before_words.is_empty() {
            if before_words.last().unwrap().is_empty() {
                before_words.pop();
            } else {
                *before_words.last_mut().unwrap() = "";
            }
            let before: String = before_words.join(" ");
            self.action_menu_cursor_x = before.chars().count();
        }
    }

    pub fn action_menu_input_right_word(&mut self) {
        let chars: Chars<'_> = self.action_menu_buffer.chars();
        let cx = self.action_menu_cursor_x;
        let after: String = chars.skip(cx).collect::<String>();
        let after_words = after.split(' ').collect::<Vec<&str>>();
        if !after_words.is_empty() {
            if after_words.first().unwrap().is_empty() {
                self.action_menu_cursor_x += 1;
            } else {
                self.action_menu_cursor_x += after_words.first().unwrap().chars().count();
            }
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
