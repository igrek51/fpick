#[cfg(test)]
mod tests {
    use std::env;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use crate::appdata::WindowFocus;
    use crate::keyboard::{handle_master_key, on_key_tree};
    use crate::filesystem::get_string_abs_path;
    
    /// Phase Red: Write a test that EXPOSES the bug
    /// The bug: First press of tilde doesn't show home directory in UI
    /// 
    /// This test simulates the full render -> handle_events -> render loop
    /// and verifies the rendered state is correct AFTER the first keypress
    #[test]
    fn test_render_state_after_first_tilde_press() {
        // Setup: Initialize app at root directory
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        app.window_focus = WindowFocus::Tree;
        
        // === First draw (iteration 1, before any keypress) ===
        // This is what the user sees BEFORE pressing tilde
        let state_before = get_string_abs_path(&app.parent_file_nodes);
        assert_eq!(state_before, "/", "initial state should be root");
        
        // === Simulate key event handling ===
        // This is what happens when user presses tilde
        let key_event = KeyEvent::new(KeyCode::Char('~'), KeyModifiers::NONE);
        
        // Step 1: handle_master_key should NOT catch tilde
        let caught_by_master = handle_master_key(&mut app, key_event);
        assert!(!caught_by_master, "tilde should NOT be caught by master_key");
        
        // Step 2: on_key_tree should route to go_to_home
        on_key_tree(&mut app, key_event);
        
        // === Verify state AFTER key handling ===
        // This is what the state SHOULD be after first press
        let state_after_key = get_string_abs_path(&app.parent_file_nodes);
        let home = env::var("HOME").expect("HOME not set");
        
        // BUG EXPOSED HERE: If this assertion fails, the bug is in state management
        assert!(
            state_after_key.starts_with(&home) || state_after_key == home,
            "BUG: After pressing tilde, state should be at home but was: {}",
            state_after_key
        );
        
        // === Verify what render would display ===
        // This simulates what the NEXT draw would show
        let title_text = app.get_current_string_path();
        
        // BUG EXPOSED HERE: If this assertion fails, the bug is in what render displays
        assert!(
            title_text.starts_with(&home) || title_text == home,
            "BUG: Render would show {} but should show {}",
            title_text,
            home
        );
        
        // === Verify child nodes for render ===
        // Render uses child_tree_nodes to build the file list
        assert!(
            !app.child_tree_nodes.is_empty(),
            "BUG: child_tree_nodes should be populated for render"
        );
        
        println!("=== BUG TEST OUTPUT ===");
        println!("Before tilde: {}", state_before);
        println!("After tilde: {}", state_after_key);
        println!("Render title: {}", title_text);
        println!("Child nodes: {} items", app.child_tree_nodes.len());
    }
    
    /// Test that verifies the ENTIRE state is consistent for rendering
    #[test]
    fn test_render_consistency_after_navigation() {
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        app.window_focus = WindowFocus::Tree;
        
        // Press tilde to go home
        let key_event = KeyEvent::new(KeyCode::Char('~'), KeyModifiers::NONE);
        
        // Simulate full handle_events flow
        if !handle_master_key(&mut app, key_event) {
            on_key_tree(&mut app, key_event);
        }
        
        // Now verify EVERYTHING render needs is in sync
        let path = get_string_abs_path(&app.parent_file_nodes);
        let title = app.get_current_string_path();
        let child_count = app.child_tree_nodes.len();
        
        // All these should be consistent
        assert_eq!(path, title, "parent_file_nodes and get_current_string_path should match");
        assert!(child_count > 0, "child_tree_nodes should have items");
        
        // The child_tree_nodes should be for the current directory
        // This is what render_dir_tree uses
        for node in &app.child_tree_nodes {
            // Just verify we can render each node - this is what render does
            let _ = node.render_list_item();
        }
    }
}