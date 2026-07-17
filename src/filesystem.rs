use anyhow::{Context, Result};
use std::fs::{self, DirEntry, ReadDir};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct FileNode {
    pub name: String,
    pub file_type: FileType,
    pub lowercase_name: String,
    pub is_symlink: bool,
    pub is_directory: bool,
    pub symlink_target: Option<String>,
    pub is_broken: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileType {
    Regular,   // regular file or symlink to regular file
    Directory, // regular directory or link to a directory
    Other,
}

pub fn list_files(dir_path: &Path) -> Result<Vec<FileNode>> {
    let dir_entries: ReadDir = fs::read_dir(dir_path)
        .with_context(|| format!("failed to read directory '{}'", dir_path.to_string_lossy()))?;

    let files: Vec<FileNode> = dir_entries
        .filter_map(|entry_r: Result<DirEntry, std::io::Error>| {
            let entry: DirEntry = entry_r.context("failed to list a file").ok()?;
            let entry_meta = fs::symlink_metadata(entry.path())
                .context("failed to read entry metadata")
                .ok()?;
            let entry_file_type = entry_meta.file_type();

            let is_symlink = entry_file_type.is_symlink();
            let mut is_directory = entry_file_type.is_dir();
            let mut is_broken = false;

            let resolved_file_type = if is_symlink {
                match fs::metadata(entry.path()) {
                    Ok(target_meta) => {
                        is_directory = target_meta.is_dir();
                        target_meta.file_type()
                    }
                    Err(_) => {
                        is_broken = true;
                        entry_file_type
                    }
                }
            } else {
                entry_file_type
            };
            let file_type = if is_directory {
                FileType::Directory
            } else if resolved_file_type.is_file() {
                FileType::Regular
            } else {
                FileType::Other
            };
            let name = entry.file_name().to_string_lossy().to_string();
            let lowercase_name = name.to_lowercase();
            let symlink_target = if is_symlink {
                std::fs::read_link(entry.path())
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
            } else {
                None
            };
            Some(FileNode {
                name,
                file_type,
                lowercase_name,
                is_symlink,
                is_directory,
                symlink_target,
                is_broken,
            })
        })
        .collect();

    return Ok(files);
}

pub fn trim_end_slash(path: String) -> String {
    if path == "/" {
        return path;
    }
    if path.ends_with('/') {
        return path[..path.len() - 1].to_string();
    }
    path.to_string()
}

pub fn normalize_path(path: String) -> String {
    path.replace("//", "/")
}

pub fn get_path_file_nodes(path: &String) -> Result<Vec<FileNode>> {
    let start_pathbuf = match path.is_empty() {
        true => PathBuf::from("."),
        false => PathBuf::from(&path),
    };
    let absolute = fs::canonicalize(&start_pathbuf).with_context(|| {
        format!(
            "evaluating absolute path '{}'",
            start_pathbuf.to_string_lossy()
        )
    })?;
    let path_parts: Vec<&str> = absolute.to_str().unwrap().split('/').collect();

    let nodes: Vec<FileNode> = path_parts
        .iter()
        .filter_map(|name: &&str| match name.len() {
            0 => None,
            _ => {
                let lowercase_name = name.to_lowercase();
                Some(FileNode {
                    name: name.to_string(),
                    file_type: FileType::Directory,
                    lowercase_name,
                    is_symlink: false,
                    is_directory: false,
                    symlink_target: None,
                    is_broken: false,
                })
            }
        })
        .collect();

    Ok(nodes)
}

pub fn get_string_abs_path(nodes: &Vec<FileNode>) -> String {
    let all_names = nodes
        .iter()
        .map(|node| node.name.to_string())
        .collect::<Vec<String>>();
    if all_names.is_empty() {
        return "/".to_string();
    }
    let path = format!("/{}", all_names.join("/"));
    normalize_path(path)
}

pub fn nodes_start_with(nodes: &Vec<FileNode>, start: &Vec<FileNode>) -> bool {
    if nodes.len() < start.len() {
        return false;
    }
    for (i, node) in start.iter().enumerate() {
        if nodes[i].name != node.name {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::TreeNodeType;
    use std::fs;
    use std::os::unix;

    static TEST_DIR_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let n = TEST_DIR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!("fpick_test_symlinks_{}", n));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();

            fs::write(dir.join("file.txt"), "hello").unwrap();
            fs::create_dir(dir.join("subdir")).unwrap();
            unix::fs::symlink("file.txt", dir.join("link_to_file")).unwrap();
            unix::fs::symlink("subdir", dir.join("link_to_dir")).unwrap();
            unix::fs::symlink("nonexistent", dir.join("broken_link")).unwrap();

            TestDir { path: dir }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn find_node<'a>(nodes: &'a [FileNode], name: &str) -> &'a FileNode {
        nodes.iter().find(|f| f.name == name).unwrap_or_else(|| panic!("node '{}' not found", name))
    }

    #[test]
    fn test_list_files_all_types() {
        let td = TestDir::new();
        let mut files = list_files(td.path()).unwrap();
        files.sort_by(|a, b| a.name.cmp(&b.name));

        let f = find_node(&files, "file.txt");
        assert_eq!(f.file_type, FileType::Regular);
        assert!(!f.is_symlink && !f.is_directory && !f.is_broken);
        assert_eq!(f.symlink_target, None);

        let f = find_node(&files, "subdir");
        assert_eq!(f.file_type, FileType::Directory);
        assert!(!f.is_symlink && f.is_directory && !f.is_broken);
        assert_eq!(f.symlink_target, None);

        let f = find_node(&files, "link_to_file");
        assert_eq!(f.file_type, FileType::Regular);
        assert!(f.is_symlink && !f.is_directory && !f.is_broken);
        assert_eq!(f.symlink_target, Some("file.txt".to_string()));

        let f = find_node(&files, "link_to_dir");
        assert_eq!(f.file_type, FileType::Directory);
        assert!(f.is_symlink && f.is_directory && !f.is_broken);
        assert_eq!(f.symlink_target, Some("subdir".to_string()));

        let f = find_node(&files, "broken_link");
        assert_eq!(f.file_type, FileType::Other);
        assert!(f.is_symlink && !f.is_directory && f.is_broken);
        assert_eq!(f.symlink_target, Some("nonexistent".to_string()));
    }

    #[test]
    fn test_full_pipeline_all_types_in_tree() {
        let td = TestDir::new();
        let mut app = crate::app::App::new();
        app.starting_dir = td.path().to_string_lossy().to_string();
        app.init().expect("init failed");
        app.window_focus = crate::appdata::WindowFocus::Tree;

        let names: Vec<&str> = app.child_tree_nodes.iter().map(|n| n.name()).collect();
        eprintln!("child_tree_nodes: {:?}", names);

        assert!(
            app.child_tree_nodes.iter().any(|n| n.name() == "file.txt"),
            "file.txt missing; got {:?}", names
        );
        assert!(
            app.child_tree_nodes.iter().any(|n| n.name() == "subdir"),
            "subdir missing; got {:?}", names
        );
        assert!(
            app.child_tree_nodes.iter().any(|n| n.name() == "link_to_file"),
            "link_to_file missing; got {:?}", names
        );
        assert!(
            app.child_tree_nodes.iter().any(|n| n.name() == "link_to_dir"),
            "link_to_dir missing; got {:?}", names
        );
        assert!(
            app.child_tree_nodes.iter().any(|n| n.name() == "broken_link"),
            "broken_link missing; got {:?}", names
        );

        assert_eq!(app.child_tree_nodes.len(), 6, "expected 6 items (. self-ref + 5 entries)");
    }

    fn render_text(file_node: &FileNode) -> String {
        let node = crate::tree::TreeNode {
            relevance: 0,
            kind: TreeNodeType::FileNode(file_node.clone()),
        };
        format!("{:?}", node.render_list_item())
    }

    #[test]
    fn test_rendering_broken_symlink_has_red_style() {
        let td = TestDir::new();
        let files = list_files(td.path()).unwrap();

        let rendered = render_text(files.iter().find(|f| f.name == "broken_link").unwrap());
        eprintln!("broken_link rendered: {}", rendered);
        assert!(rendered.contains("broken_link"), "filename in output");
        assert!(rendered.contains("@"), "symlink marker");
        assert!(rendered.contains("nonexistent"), "symlink target");
        assert!(rendered.contains("light_red"), "light red style");
        assert!(rendered.contains("bold"), "bold style");
        assert!(!rendered.contains("(broken)"), "no (broken) text suffix");
    }

    #[test]
    fn test_rendering_symlink_to_file_has_no_broken_marker() {
        let td = TestDir::new();
        let files = list_files(td.path()).unwrap();

        let rendered = render_text(files.iter().find(|f| f.name == "link_to_file").unwrap());
        eprintln!("link_to_file rendered: {}", rendered);
        assert!(rendered.contains("link_to_file"));
        assert!(rendered.contains("@"));
        assert!(rendered.contains("file.txt"));
        assert!(!rendered.contains("(broken)"), "no (broken) text");
        assert!(rendered.contains("light_cyan"), "light cyan style");
    }

    #[test]
    fn test_rendering_symlink_to_dir_ends_with_slash() {
        let td = TestDir::new();
        let files = list_files(td.path()).unwrap();

        let rendered = render_text(files.iter().find(|f| f.name == "link_to_dir").unwrap());
        eprintln!("link_to_dir rendered: {}", rendered);
        assert!(rendered.contains("link_to_dir"));
        assert!(rendered.contains("@"));
        assert!(rendered.contains("subdir"));
        assert!(rendered.contains("/"), "dir symlink ends with /");
        assert!(rendered.contains("light_blue"), "directory style");
    }

    #[test]
    fn test_rendering_regular_file_no_symlink_markers() {
        let td = TestDir::new();
        let files = list_files(td.path()).unwrap();

        let rendered = render_text(files.iter().find(|f| f.name == "file.txt").unwrap());
        eprintln!("file.txt rendered: {}", rendered);
        assert!(rendered.contains("file.txt"));
        assert!(!rendered.contains("@"), "no symlink marker");
        assert!(!rendered.contains("⇒"), "no arrow");
    }

    #[test]
    fn test_rendering_regular_directory_ends_with_slash() {
        let td = TestDir::new();
        let files = list_files(td.path()).unwrap();

        let rendered = render_text(files.iter().find(|f| f.name == "subdir").unwrap());
        eprintln!("subdir rendered: {}", rendered);
        assert!(rendered.contains("subdir"));
        assert!(rendered.contains("/"), "directory ends with /");
        assert!(!rendered.contains("@"), "no symlink marker");
    }

    #[test]
    fn test_e2e_app_lifecycle_broken_symlink_present() {
        let td = TestDir::new();
        let mut app = crate::app::App::new();
        app.starting_dir = td.path().to_string_lossy().to_string();
        app.init().expect("init failed");

        let names: Vec<&str> = app.child_tree_nodes.iter().map(|n| n.name()).collect();
        eprintln!("e2e child_tree_nodes: {:?}", names);

        assert!(
            app.child_tree_nodes.iter().any(|n| n.name() == "broken_link"),
            "broken_link NOT in child_tree_nodes after full App::init!\nGot: {:?}",
            names
        );

        let broken_node = app.child_tree_nodes.iter().find(|n| n.name() == "broken_link").unwrap();
        let rendered = format!("{:?}", broken_node.render_list_item());
        assert!(rendered.contains("light_red"), "broken link should be light red: {}", rendered);
        assert!(!rendered.contains("(broken)"), "no (broken) suffix: {}", rendered);
    }
}
