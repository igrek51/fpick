use anyhow::{Context, Result};
use std::fs::{self, symlink_metadata, ReadDir};
use std::fs::{metadata, DirEntry};
use std::path::Path;

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
    let dir_entries: ReadDir = fs::read_dir(dir_path).context("failed to read directory")?;

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
