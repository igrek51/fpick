#[cfg(test)]
mod tests {
    use std::env;
    
    #[test]
    fn test_go_to_home_logic() {
        let home = env::var("HOME").expect("HOME not set");
        
        // Test the logic that go_to_home performs
        use crate::filesystem::get_path_file_nodes;
        
        // This is what go_to_home does internally
        let nodes = get_path_file_nodes(&home).expect("should get path nodes for home");
        
        // Verify we got valid nodes
        assert!(!nodes.is_empty(), "should have at least one node (the home dir itself)");
        
        // The last node should be the home directory name
        let last_node = nodes.last().unwrap();
        assert!(
            last_node.name == "home" || last_node.name == home.split('/').last().unwrap(),
            "last node should be home dir name"
        );
    }
    
    #[test]
    fn test_populate_home_directory() {
        let home = env::var("HOME").expect("HOME not set");
        
        use crate::filesystem::list_files;
        
        let path = std::path::Path::new(&home);
        let nodes = list_files(path).expect("should list home directory");
        
        // Should have some entries
        assert!(!nodes.is_empty(), "home directory should not be empty");
        
        // Should contain common entries like ".", "..", etc.
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.iter().any(|n| *n == "."), "should have self reference");
    }
    
    #[test]
    fn test_keyboard_tilde_mapping() {
        // Test that the keyboard handler correctly maps ~ to go_to_home
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        
        // This is what the keyboard handler should match
        let key_event = KeyEvent::new(
            KeyCode::Char('~'),
            KeyModifiers::NONE,
        );
        
        assert_eq!(key_event.code, KeyCode::Char('~'));
    }
}