#[cfg(test)]
mod tests {
    use std::env;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use crate::appdata::WindowFocus;
    use crate::keyboard::handle_master_key;
    use crate::filesystem::get_string_abs_path;
    
    #[test]
    fn test_full_key_event_loop_key_handler() {
        // Simulate the full key event loop:
        // 1. handle_master_key is called first
        // 2. If it returns false, on_key_tree is called
        // 3. on_key_tree should route tilde to go_to_home
        
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        app.window_focus = WindowFocus::Tree;
        
        let key_event = KeyEvent::new(KeyCode::Char('~'), KeyModifiers::NONE);
        
        // Step 1: handle_master_key
        let master_handled = handle_master_key(&mut app, key_event);
        
        // handle_master_key should return false for tilde (since no error/info)
        assert!(!master_handled, "tilde should NOT be handled by master_key");
        
        // Step 2: Since master_key returned false, we should call on_key_tree
        // This is what should happen in update_on_key
        use crate::keyboard::on_key_tree;
        on_key_tree(&mut app, key_event);
        
        // Step 3: Verify state changed
        let new_path = get_string_abs_path(&app.parent_file_nodes);
        let home = env::var("HOME").expect("HOME not set");
        
        assert!(
            new_path.starts_with(&home),
            "on_key_tree should call go_to_home for tilde"
        );
    }
    
    #[test]
    fn test_key_routing_no_error_state() {
        // Test: when there is NO error, tilde should go to on_key_tree
        
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        
        // Ensure no error message
        assert!(!app.has_error(), "should have no error");
        assert!(!app.has_info(), "should have no info");
        
        let key_event = KeyEvent::new(KeyCode::Char('~'), KeyModifiers::NONE);
        
        let result = handle_master_key(&mut app, key_event);
        
        // tilde should NOT be handled by master_key when no error
        assert!(!result, "master_key should return false for tilde");
    }
    
    #[test]
    fn test_key_routing_with_error_state() {
        // Test: when there IS error, Enter should be handled by master_key
        
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        
        // Set an error
        app.show_error("test error".to_string());
        
        assert!(app.has_error(), "should have error");
        
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        
        let result = handle_master_key(&mut app, key_event);
        
        // Enter with error should be handled by master_key (clear error)
        assert!(result, "master_key should return true for Enter when error");
        assert!(!app.has_error(), "error should be cleared");
    }
    
    #[test]
    fn test_key_routing_with_info_state() {
        // Test: when there IS info, Up/Down should be handled by master_key
        
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        
        // Set info
        app.show_info("test info".to_string());
        
        assert!(app.has_info(), "should have info");
        
        // Down with info
        let key_event = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let result = handle_master_key(&mut app, key_event);
        
        assert!(result, "master_key should return true for Down when info");
    }
}