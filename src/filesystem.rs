use anyhow::{Context, Result};
use std::fs::{self, symlink_metadata, ReadDir};
use std::fs::{metadata, DirEntry};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
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
        .filter_map(|entry| {
            let entry: DirEntry = entry.context("failed to list a file").ok()?;
            let md = metadata(entry.path())
                .context("failed to read file metadata")
                .ok()?;
            let symlink_md = symlink_metadata(entry.path())
                .context("failed to read symlink metadata")
                .ok()?;
            let is_symlink = symlink_md.is_symlink();
            let is_directory = md.is_dir();
            let file_type = if is_directory {
                FileType::Directory
            } else if md.is_file() {
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
