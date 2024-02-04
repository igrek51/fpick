use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit()
        }
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::Left => app.go_up(),
        KeyCode::Right => app.go_into(),
        KeyCode::Enter => app.pick_file(),
        KeyCode::PageDown => app.move_cursor(20),
        KeyCode::PageUp => app.move_cursor(-20),
        KeyCode::Home => app.move_cursor(-(app.child_nodes.len() as i32)),
        KeyCode::End => app.move_cursor(app.child_nodes.len() as i32),
        KeyCode::Char('u') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.clear_search_text();
        }
        KeyCode::Backspace => {
            app.backspace_search_text();
        }
        KeyCode::Char('w') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.backspace_search_text();
        }
        KeyCode::Char(c) => {
            app.type_search_text(c);
        }
        KeyCode::F(5) => {
            app.populate_current_child_nodes();
        }
        _ => {}
    };
}
