#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::app::App;
    use crate::filesystem::get_string_abs_path;

    static TEST_DIR_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let n = TEST_DIR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!("fpick_start_path_test_{}", n));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(dir.join("subdir")).unwrap();
            fs::write(dir.join("a.txt"), "a").unwrap();
            fs::write(dir.join("b.txt"), "b").unwrap();
            fs::write(dir.join("c.txt"), "c").unwrap();
            TestDir { path: dir }
        }

        fn path(&self) -> &PathBuf {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn canonical(path: &PathBuf) -> String {
        fs::canonicalize(path)
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    fn app_at(path: &PathBuf) -> App {
        let mut app = App::new();
        app.starting_dir = path.to_string_lossy().to_string();
        app
    }

    #[test]
    fn test_start_on_file_focuses_it_in_parent_dir() {
        let td = TestDir::new();
        let file = td.path().join("b.txt");
        let mut app = app_at(&file);

        app.init().expect("init failed");

        assert_eq!(
            get_string_abs_path(&app.parent_file_nodes),
            canonical(td.path()),
            "should start in the file's parent directory"
        );
        assert_eq!(
            app.get_selected_tree_node().map(|node| node.name().to_string()),
            Some("b.txt".to_string()),
            "file should be focused"
        );
        assert!(app.dir_cursor > 0, "self-reference should not be focused");
        assert_eq!(
            app.dir_cursor,
            app.child_tree_nodes
                .iter()
                .position(|node| node.name() == "b.txt")
                .unwrap()
        );
    }

    #[test]
    fn test_start_on_directory_keeps_self_reference_focused() {
        let td = TestDir::new();
        let mut app = app_at(td.path());

        app.init().expect("init failed");

        assert_eq!(
            get_string_abs_path(&app.parent_file_nodes),
            canonical(td.path())
        );
        assert_eq!(app.dir_cursor, 0);
        assert_eq!(
            app.get_selected_tree_node().map(|node| node.name().to_string()),
            Some(".".to_string())
        );
    }

    #[test]
    fn test_enter_on_focused_file_picks_it() {
        let td = TestDir::new();
        let file = td.path().join("b.txt");
        let mut app = app_at(&file);
        app.init().expect("init failed");

        app.enter_selected_node();

        assert_eq!(app.picked_path, Some(canonical(&file)));
        assert!(app.should_quit);
    }

    #[test]
    fn test_relative_path_uses_parent_dir_when_started_on_file() {
        let td = TestDir::new();
        let file = td.path().join("b.txt");
        let mut app = app_at(&file);
        app.relative_path = true;
        app.init().expect("init failed");

        app.pick_selected_node(None);

        assert_eq!(app.picked_path.as_deref(), Some("b.txt"));
        assert!(app.should_quit);
    }

    #[test]
    fn test_start_on_missing_path_errors() {
        let td = TestDir::new();
        let missing = td.path().join("nope.txt");
        let mut app = app_at(&missing);

        assert!(app.init().is_err());
    }
}
