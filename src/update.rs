use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::App, appdata::WindowFocus, logs::log, tui::Tui};

pub fn update(app: &mut App, key_event: KeyEvent, tui: &mut Tui) {
    match app.window_focus {
        WindowFocus::Tree => on_key_tree(app, key_event),
        WindowFocus::ActionMenu => on_key_action_menu(app, key_event, tui),
        WindowFocus::ActionMenuStep2 => on_key_action_menu_step2(app, key_event, tui),
    }
}

pub fn on_key_tree(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Enter | KeyCode::Esc if app.has_error() => app.clear_error(),
        KeyCode::Enter | KeyCode::Esc if app.has_info() => app.clear_info(),
        KeyCode::Esc if !app.filter_text.is_empty() => app.clear_search_text(),
        KeyCode::Esc => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit();
        }
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::Left => app.go_up(),
        KeyCode::Char('/') => app.go_to_root(),
        KeyCode::Right | KeyCode::Tab => app.go_into(),
        KeyCode::Enter if key_event.modifiers == KeyModifiers::ALT => app.open_action_dialog(),
        KeyCode::Enter => app.enter_selected_node(),
        KeyCode::Char('>') => app.pick_current_dir(), // Shift + .
        KeyCode::F(5) => app.populate_current_child_nodes(),
        KeyCode::PageDown => app.move_cursor(20),
        KeyCode::PageUp => app.move_cursor(-20),
        KeyCode::Home => app.move_cursor(-(app.child_file_nodes.len() as i32)),
        KeyCode::End => app.move_cursor(app.child_file_nodes.len() as i32),
        KeyCode::Char('u') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.clear_search_text();
        }
        KeyCode::Backspace => app.backspace_search_text(),
        KeyCode::Char('w') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.backspace_search_text();
        }
        KeyCode::Char(c) => app.type_search_text(c),
        _ => log(format!("Unknown key event: {:?}", key_event).as_str()),
    };
}

pub fn on_key_action_menu(app: &mut App, key_event: KeyEvent, tui: &mut Tui) {
    match key_event.code {
        KeyCode::Enter | KeyCode::Esc if app.has_error() => app.clear_error(),
        KeyCode::Enter | KeyCode::Esc if app.has_info() => app.clear_info(),
        KeyCode::Esc => app.close_action_dialog(),
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit();
        }
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::Home => app.move_cursor(-(app.known_menu_actions.len() as i32)),
        KeyCode::End => app.move_cursor(app.known_menu_actions.len() as i32),
        KeyCode::Enter => app.execute_dialog_action(tui),
        _ => {
            log(format!("Unknown key event: {:?}", key_event).as_str());
        }
    };
}

pub fn on_key_action_menu_step2(app: &mut App, key_event: KeyEvent, tui: &mut Tui) {
    match key_event.code {
        KeyCode::Enter | KeyCode::Esc if app.has_error() => app.clear_error(),
        KeyCode::Enter | KeyCode::Esc if app.has_info() => app.clear_info(),
        KeyCode::Esc => app.close_action_dialog(),
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit();
        }
        KeyCode::Enter => app.execute_dialog_action_step2(tui),
        KeyCode::Char('u') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.action_menu_input_clear_backwards();
        }
        KeyCode::Backspace => app.action_menu_input_backspace(),
        KeyCode::Delete => app.action_menu_input_delete(),
        KeyCode::Char('w') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.action_menu_input_backspace();
        }
        KeyCode::Char(c) => app.action_menu_input_append(c),
        KeyCode::Left => app.action_menu_input_left(),
        KeyCode::Right => app.action_menu_input_right(),
        KeyCode::Home => app.action_menu_input_home(),
        KeyCode::End => app.action_menu_input_end(),
        _ => {
            log(format!("Unknown key event: {:?}", key_event).as_str());
        }
    };
}
