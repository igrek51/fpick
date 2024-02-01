use anyhow::{Context, Result};
use std::fs::{self, ReadDir};
use std::fs::{metadata, DirEntry};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub file_type: FileType,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileType {
    Regular,
    Directory,
    Link,
    Other,
}

impl FileNode {
    pub fn display_name(&self) -> String {
        match self.file_type {
            FileType::Directory => format!("{}/", self.name),
            FileType::Link => format!("{}@", self.name),
            _ => self.name.clone(),
        }
    }
}

pub fn list_files(dir_path: &Path) -> Result<Vec<FileNode>> {
    let dir_entries: ReadDir = fs::read_dir(dir_path).context("failed to read directory")?;

    let files: Vec<FileNode> = dir_entries
        .filter_map(|entry| {
            let entry: DirEntry = entry.context("failed to list a file").ok()?;
            let md = metadata(entry.path())
                .context("failed to read file metadata")
                .ok()?;
            let file_type = if md.is_dir() {
                FileType::Directory
            } else if md.is_file() {
                FileType::Regular
            } else if md.file_type().is_symlink() {
                FileType::Link
            } else {
                FileType::Other
            };
            let name = entry.file_name().to_string_lossy().to_string();
            Some(FileNode { name, file_type })
        })
        .collect();

    return Ok(files);
}
