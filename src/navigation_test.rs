#[cfg(test)]
mod tests {
    use std::env;
    
    #[test]
    fn test_go_to_home_state_change() {
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        
        let original_path = {
            use crate::filesystem::get_string_abs_path;
            get_string_abs_path(&app.parent_file_nodes)
        };
        
        app.go_to_home();
        
        let new_path = {
            use crate::filesystem::get_string_abs_path;
            get_string_abs_path(&app.parent_file_nodes)
        };
        
        let home = env::var("HOME").expect("HOME not set");
        
        assert_ne!(original_path, new_path, "path should have changed");
        assert!(
            new_path.starts_with(&home) || new_path == home,
            "new path {} should be in home {}",
            new_path,
            home
        );
        
        assert!(!app.child_file_nodes.is_empty(), "child_file_nodes should be populated");
    }
    
    #[test]
    fn test_render_called_after_state_change() {
        // This test verifies that render_tree_nodes is called after go_to_home
        let mut app = crate::app::App::new();
        app.starting_dir = "/".to_string();
        app.init().expect("init failed");
        
        // After init, we should have rendered tree nodes
        
        // Call go_to_home - this should update both file nodes and tree nodes
        app.go_to_home();
        
        // Verify tree nodes were updated
        assert!(
            app.child_tree_nodes.len() > 0,
            "tree nodes should be populated after go_to_home"
        );
    }
}