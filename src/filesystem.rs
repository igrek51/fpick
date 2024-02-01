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
}

impl FileNode {
    pub fn display_name(&self) -> String {
        let mut display = self.name.clone();
        if self.is_symlink {
            display = format!("{}@", display)
        }
        if self.file_type == FileType::Directory {
            display = format!("{}/", display)
        }
        display
    }
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
            let file_type = if md.is_dir() {
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
            })
        })
        .collect();

    return Ok(files);
}
