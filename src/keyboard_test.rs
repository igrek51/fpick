#[cfg(test)]
mod tests {
    use std::env;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use crate::appdata::WindowFocus;
    use crate::keyboard::handle_master_key;
    
    #[test]
    fn test_handle_master_key_returns_false_for_tilde() {
        let mut app = crate::app::App::new();
        app.init().expect("init failed");
        
        let key_event = KeyEvent::new(KeyCode::Char('~'), KeyModifiers::NONE);
        
        let result = handle_master_key(&mut app, key_event);
        
        assert!(!result, "handle_master_key should return false for tilde key (no error/info state)");
    }
    
    #[test]
    fn test_update_on_key_routes_tilde_to_go_to_home() {
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        app.window_focus = WindowFocus::Tree;
        
        let original_path = {
            use crate::filesystem::get_string_abs_path;
            get_string_abs_path(&app.parent_file_nodes)
        };
        
        let key_event = KeyEvent::new(KeyCode::Char('~'), KeyModifiers::NONE);
        
        use crate::keyboard::on_key_tree;
        on_key_tree(&mut app, key_event);
        
        let new_path = {
            use crate::filesystem::get_string_abs_path;
            get_string_abs_path(&app.parent_file_nodes)
        };
        
        let home = env::var("HOME").expect("HOME not set");
        
        assert_ne!(original_path, new_path, "path should change after pressing tilde");
        assert!(
            new_path.starts_with(&home),
            "new path {} should start with home {}",
            new_path,
            home
        );
    }
    
    #[test]
    fn test_keyboard_handler_tilde_with_shift_modifier() {
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        app.window_focus = WindowFocus::Tree;
        
        let key_event = KeyEvent::new(KeyCode::Char('~'), KeyModifiers::SHIFT);
        
        use crate::keyboard::on_key_tree;
        on_key_tree(&mut app, key_event);
        
        let new_path = {
            use crate::filesystem::get_string_abs_path;
            get_string_abs_path(&app.parent_file_nodes)
        };
        
        let home = env::var("HOME").expect("HOME not set");
        
        assert!(
            new_path.starts_with(&home),
            "tilde with SHIFT should also navigate to home: got {}",
            new_path
        );
    }
    
    #[test]
    fn test_backtick_key_navigates_to_home() {
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        app.window_focus = WindowFocus::Tree;
        
        let key_event = KeyEvent::new(KeyCode::Char('`'), KeyModifiers::NONE);
        
        use crate::keyboard::on_key_tree;
        on_key_tree(&mut app, key_event);
        
        let new_path = {
            use crate::filesystem::get_string_abs_path;
            get_string_abs_path(&app.parent_file_nodes)
        };
        
        let home = env::var("HOME").expect("HOME not set");
        
        assert!(
            new_path.starts_with(&home),
            "backtick should navigate to home: got {}",
            new_path
        );
    }
    
    #[test]
    fn test_slash_key_navigates_to_root() {
        let mut app = crate::app::App::new();
        let home = env::var("HOME").expect("HOME not set");
        app.starting_dir = home.clone();
        app.init().expect("init failed");
        app.window_focus = WindowFocus::Tree;
        
        let key_event = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        
        use crate::keyboard::on_key_tree;
        on_key_tree(&mut app, key_event);
        
        let new_path = {
            use crate::filesystem::get_string_abs_path;
            get_string_abs_path(&app.parent_file_nodes)
        };
        
        assert_eq!(new_path, "/", "slash should navigate to root");
    }
}