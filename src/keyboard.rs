use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::App, appdata::WindowFocus, logs::log, tui::Tui};

pub fn update_on_key(app: &mut App, key_event: KeyEvent, tui: &mut Tui) {
    if handle_master_key(app, key_event) {
        return;
    }
    match app.window_focus {
        WindowFocus::Tree => on_key_tree(app, key_event),
        WindowFocus::ActionMenu => on_key_action_menu(app, key_event, tui),
        WindowFocus::ActionMenuStep2 => on_key_action_menu_step2(app, key_event, tui),
    }
}

pub fn handle_master_key(app: &mut App, key_event: KeyEvent) -> bool {
    match key_event.code {
        KeyCode::Enter | KeyCode::Esc if app.has_error() => app.clear_error(),
        KeyCode::Enter | KeyCode::Esc if app.has_info() => app.clear_info(),
        KeyCode::Char('c') | KeyCode::Char('C') if is_ctrl(key_event) => app.quit(),
        KeyCode::Down if app.has_info() => app.move_cursor(1),
        KeyCode::Up if app.has_info() => app.move_cursor(-1),
        KeyCode::Left | KeyCode::Right if app.has_info() => return true,
        _ => return false,
    };
    true
}

pub fn on_key_tree(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc if !app.filter_text.is_empty() => app.clear_search_text(),
        KeyCode::Esc => app.quit(),
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::Left => app.go_up(),
        KeyCode::Char('/') => app.go_to_root(),
        KeyCode::Right | KeyCode::Tab => app.go_into(),
        KeyCode::Enter if key_event.modifiers == KeyModifiers::ALT => app.open_action_dialog(),
        KeyCode::Enter if key_event.modifiers == KeyModifiers::SHIFT => app.open_action_dialog(),
        KeyCode::Enter if key_event.modifiers == KeyModifiers::SUPER => app.open_action_dialog(),
        KeyCode::Enter if key_event.modifiers == KeyModifiers::META => app.open_action_dialog(),
        KeyCode::Enter if key_event.modifiers == KeyModifiers::CONTROL => app.open_action_dialog(),
        KeyCode::Enter => app.enter_selected_node(),
        KeyCode::Char('>') => app.pick_current_dir(), // Shift + .
        KeyCode::F(2) => app.rename_selected_node(),
        KeyCode::F(4) => app.open_action_dialog(),
        KeyCode::F(5) => app.populate_current_child_nodes(),
        KeyCode::PageDown => app.move_cursor(20),
        KeyCode::PageUp => app.move_cursor(-20),
        KeyCode::Home => app.move_cursor(-(app.child_file_nodes.len() as i32)),
        KeyCode::End => app.move_cursor(app.child_file_nodes.len() as i32),
        KeyCode::Char('r') if is_ctrl(key_event) => app.rename_selected_node(),
        KeyCode::Char('d') if is_ctrl(key_event) => app.delete_selected_node_confirm(),
        KeyCode::Char('u') if is_ctrl(key_event) => app.clear_search_text(),
        KeyCode::Char('w') if is_ctrl(key_event) => app.backspace_search_text(),
        KeyCode::Char('o') if is_ctrl(key_event) => app.open_action_dialog(),
        KeyCode::Backspace => app.backspace_search_text(),
        KeyCode::Char(c) => app.type_search_text(c),
        _ => log(format!("Unknown key event: {:?}", key_event).as_str()),
    };
}

pub fn on_key_action_menu(app: &mut App, key_event: KeyEvent, tui: &mut Tui) {
    match key_event.code {
        KeyCode::Esc => app.close_action_dialog(),
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::PageDown => app.move_cursor(20),
        KeyCode::PageUp => app.move_cursor(-20),
        KeyCode::Home => app.move_cursor(-(app.known_menu_actions.len() as i32)),
        KeyCode::End => app.move_cursor(app.known_menu_actions.len() as i32),
        KeyCode::Enter => app.execute_dialog_action(tui),
        _ => log(format!("Unknown key event: {:?}", key_event).as_str()),
    };
}

pub fn on_key_action_menu_step2(app: &mut App, key_event: KeyEvent, tui: &mut Tui) {
    match key_event.code {
        KeyCode::Esc => app.close_action_dialog(),
        KeyCode::Enter => app.execute_dialog_action_step2(tui),
        KeyCode::Char('u') if is_ctrl(key_event) => app.action_menu_input_clear_backwards(),
        KeyCode::Char('k') if is_ctrl(key_event) => app.action_menu_input_clear_forward(),
        KeyCode::Char('w') if is_ctrl(key_event) => app.action_menu_input_backspace_word(),
        KeyCode::Backspace if is_ctrl(key_event) => app.action_menu_input_backspace_word(),
        KeyCode::Backspace if is_alt(key_event) => app.action_menu_input_backspace_word(),
        KeyCode::Delete if is_ctrl(key_event) => app.action_menu_input_delete_word(),
        KeyCode::Backspace => app.action_menu_input_backspace(),
        KeyCode::Delete => app.action_menu_input_delete(),
        KeyCode::Char(c) => app.action_menu_input_append(c),
        KeyCode::Left if is_ctrl(key_event) => app.action_menu_input_left_word(),
        KeyCode::Right if is_ctrl(key_event) => app.action_menu_input_right_word(),
        KeyCode::Left => app.action_menu_input_left(),
        KeyCode::Right => app.action_menu_input_right(),
        KeyCode::Home => app.action_menu_input_home(),
        KeyCode::End => app.action_menu_input_end(),
        _ => log(format!("Unknown key event: {:?}", key_event).as_str()),
    };
}

fn is_ctrl(key_event: KeyEvent) -> bool {
    key_event.modifiers == KeyModifiers::CONTROL
}

fn is_alt(key_event: KeyEvent) -> bool {
    key_event.modifiers == KeyModifiers::ALT
}
