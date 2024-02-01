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
            app.filter_text.clear();
            app.render_tree_nodes();
        }
        KeyCode::Backspace => {
            app.filter_text.pop();
            app.render_tree_nodes();
        }
        KeyCode::Char('w') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.filter_text.pop();
            app.render_tree_nodes();
        }
        KeyCode::Char(c) => {
            app.filter_text.push(c);
            app.render_tree_nodes();
        }
        KeyCode::F(5) => {
            app.populate_current_child_nodes();
        }
        _ => {}
    };
}
