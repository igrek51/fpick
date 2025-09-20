use anyhow::{Context, Result};
use std::fs::{self, DirEntry, Metadata, ReadDir};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct FileNode {
    pub name: String,
    pub file_type: FileType,
    pub lowercase_name: String,
    pub is_symlink: bool,
    pub is_directory: bool,
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
            let file_type = entry
                .file_type()
                .context("failed to check the file type")
                .ok()?;

            let is_symlink = file_type.is_symlink();
            let mut is_directory = file_type.is_dir();

            let resolved_file_type = if is_symlink {
                let symlink_md: Metadata = fs::metadata(entry.path())
                    .context("failed to read symlink metadata")
                    .ok()?;
                is_directory = symlink_md.is_dir();
                symlink_md.file_type()
            } else {
                file_type
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
            Some(FileNode {
                name,
                file_type,
                lowercase_name,
                is_symlink,
                is_directory,
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
