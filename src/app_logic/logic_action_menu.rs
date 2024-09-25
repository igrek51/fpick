use crate::action_menu::{
    create_directory, create_file, execute_shell_operation, rename_file, MenuAction, Operation,
};
use crate::app::App;
use crate::appdata::WindowFocus;
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

    pub fn execute_dialog_action(&mut self, _: &mut Tui) {
        let path = self.get_selected_abs_path();
        if path.is_none() {
            return;
        }

        let action: &MenuAction = &self.known_menu_actions[self.action_menu_cursor_y];

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
                self.action_menu_operation = Some(action.operation.clone());
                self.action_menu_title = format!("New name for {}", filename);
                self.action_menu_buffer = filename;
                self.action_menu_cursor_x = self.action_menu_buffer.len();
            }
            Operation::CreateFile => {
                let current_dir_path: String = self.get_current_dir_abs_path();
                self.window_focus = WindowFocus::ActionMenuStep2;
                self.action_menu_operation = Some(action.operation.clone());
                self.action_menu_title = format!("New file at {}", current_dir_path);
                self.action_menu_buffer = "".to_string();
                self.action_menu_cursor_x = self.action_menu_buffer.len();
            }
            Operation::CreateDir => {
                let current_dir_path: String = self.get_current_dir_abs_path();
                self.window_focus = WindowFocus::ActionMenuStep2;
                self.action_menu_operation = Some(action.operation.clone());
                self.action_menu_title = format!("New directory at {}", current_dir_path);
                self.action_menu_buffer = "".to_string();
                self.action_menu_cursor_x = self.action_menu_buffer.len();
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
        let before = self.action_menu_buffer[..self.action_menu_cursor_x].to_string();
        let after = self.action_menu_buffer[self.action_menu_cursor_x..].to_string();
        self.action_menu_buffer = before + &c.to_string() + &after;
        self.action_menu_input_right();
    }

    pub fn action_menu_input_clear_backwards(&mut self) {
        let after = self.action_menu_buffer[self.action_menu_cursor_x..].to_string();
        self.action_menu_buffer = after;
        self.action_menu_cursor_x = 0;
    }

    pub fn action_menu_input_backspace(&mut self) {
        let mut before = self.action_menu_buffer[..self.action_menu_cursor_x].to_string();
        let after = self.action_menu_buffer[self.action_menu_cursor_x..].to_string();
        if !before.is_empty() {
            before.pop();
            self.action_menu_buffer = before + &after;
            self.action_menu_input_left();
        }
    }

    pub fn action_menu_input_delete(&mut self) {
        let before = self.action_menu_buffer[..self.action_menu_cursor_x].to_string();
        let mut after = self.action_menu_buffer[self.action_menu_cursor_x..].to_string();
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
        if self.action_menu_cursor_x > self.action_menu_buffer.len() {
            self.action_menu_cursor_x = self.action_menu_buffer.len();
        }
    }

    pub fn action_menu_input_home(&mut self) {
        self.action_menu_cursor_x = 0;
    }

    pub fn action_menu_input_end(&mut self) {
        self.action_menu_cursor_x = self.action_menu_buffer.len();
    }
}
