#[cfg(test)]
mod tests {
    use std::env;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use crate::appdata::WindowFocus;
    use crate::keyboard::{handle_master_key, on_key_tree};
    use crate::filesystem::get_string_abs_path;
    
    #[test]
    fn test_full_loop_simulation() {
        // Simulate the full main loop: draw -> handle_events -> draw
        // This tests the actual user flow
        
        // Initialize app at root
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        app.window_focus = WindowFocus::Tree;
        
        // === FIRST ITERATION ===
        // Draw 1: shows current state (root)
        let state_after_init = get_string_abs_path(&app.parent_file_nodes);
        assert_eq!(state_after_init, "/", "initial state should be root");
        
        // Before: draw shows root
        // Now: press tilde (simulate key event)
        let key_event = KeyEvent::new(KeyCode::Char('~'), KeyModifiers::NONE);
        
        // Step 1: handle_master_key should return false
        let master_result = handle_master_key(&mut app, key_event);
        assert!(!master_result, "tilde should NOT be handled by master_key");
        
        // Step 2: on_key_tree handles the key
        on_key_tree(&mut app, key_event);
        
        // After handle_events: state should be updated to HOME
        let state_after_key = get_string_abs_path(&app.parent_file_nodes);
        let home = env::var("HOME").expect("HOME not set");
        
        assert!(
            state_after_key.starts_with(&home) || state_after_key == home,
            "after pressing tilde, state should be home: got {}",
            state_after_key
        );
        
        // === SECOND ITERATION ===
        // Draw 2: should show home (the new state)
        // This is what the user should see after first keypress
        
        println!("=== FULL LOOP SIMULATION ===");
        println!("Initial state (root): {}", state_after_init);
        println!("After pressing ~: {}", state_after_key);
        println!("=== EXPECTED: User sees home after first press ===");
    }
    
    #[test]
    fn test_key_vs_determinism() {
        // Test that the key handling is deterministic
        // Same key press should always produce same result
        
        let mut app1 = crate::app::App::new();
        app1.starting_dir = "/".to_string();
        app1.init().expect("init failed");
        app1.window_focus = WindowFocus::Tree;
        
        let key_event = KeyEvent::new(KeyCode::Char('~'), KeyModifiers::NONE);
        on_key_tree(&mut app1, key_event);
        let path1 = get_string_abs_path(&app1.parent_file_nodes);
        
        // New app instance - same result
        let mut app2 = crate::app::App::new();
        app2.starting_dir = "/".to_string();
        app2.init().expect("init failed");
        app2.window_focus = WindowFocus::Tree;
        
        on_key_tree(&mut app2, key_event);
        let path2 = get_string_abs_path(&app2.parent_file_nodes);
        
        assert_eq!(path1, path2, "key handling should be deterministic");
    }
}