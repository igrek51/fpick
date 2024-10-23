use anyhow::{anyhow, Context, Ok, Result};
use arboard::Clipboard;
use chrono::prelude::{DateTime, Utc};
use std::{
    fs,
    process::{Command, ExitStatus, Stdio},
};

use crate::{
    filesystem::FileType,
    logs::log,
    tree::{TreeNode, TreeNodeType},
    tui::Tui,
};

#[derive(Debug, Clone)]
pub struct MenuAction {
    pub name: &'static str,
    pub operation: Operation,
}

#[derive(Debug, Clone)]
pub enum Operation {
    ShellCommand { template: &'static str },
    InteractiveShellCommand { template: &'static str },
    PickAbsolutePath,
    PickRelativePath,
    Rename,
    CreateFile,
    CreateDir,
    Delete,
    CopyToClipboard { is_relative_path: bool },
    FileDetails,
    CustomCommand,
}

pub fn generate_known_actions() -> Vec<MenuAction> {
    vec![
        MenuAction {
            name: "Open",
            operation: Operation::ShellCommand {
                template: "xdg-open \"{}\"",
            },
        },
        MenuAction {
            name: "Show in less",
            operation: Operation::InteractiveShellCommand {
                template: "less \"{}\"",
            },
        },
        MenuAction {
            name: "Edit in vim",
            operation: Operation::InteractiveShellCommand {
                template: "vim \"{}\"",
            },
        },
        MenuAction {
            name: "Edit in sudo vim",
            operation: Operation::InteractiveShellCommand {
                template: "sudo vim \"{}\"",
            },
        },
        MenuAction {
            name: "Delete",
            operation: Operation::Delete,
        },
        MenuAction {
            name: "Copy relative path to clipboard",
            operation: Operation::CopyToClipboard {
                is_relative_path: true,
            },
        },
        MenuAction {
            name: "Copy absolute path to clipboard",
            operation: Operation::CopyToClipboard {
                is_relative_path: false,
            },
        },
        MenuAction {
            name: "Rename",
            operation: Operation::Rename,
        },
        MenuAction {
            name: "Create file",
            operation: Operation::CreateFile,
        },
        MenuAction {
            name: "Create directory",
            operation: Operation::CreateDir,
        },
        MenuAction {
            name: "Pick absolute path",
            operation: Operation::PickAbsolutePath,
        },
        MenuAction {
            name: "Pick relative path",
            operation: Operation::PickRelativePath,
        },
        MenuAction {
            name: "Details",
            operation: Operation::FileDetails,
        },
        MenuAction {
            name: "Run command",
            operation: Operation::CustomCommand,
        },
    ]
}

pub fn execute_shell_operation(path: &String, command_template: &str) -> Result<()> {
    let cmd = String::from(command_template).replace("{}", path);
    execute_shell(cmd.clone())
}

pub fn execute_interactive_shell_operation(
    path: &String,
    command_template: &str,
    tui: &mut Tui,
) -> Result<()> {
    let cmd = String::from(command_template).replace("{}", path);
    tui.exit().context("failed to exit TUI mode")?;
    let mut output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd.clone())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to start a command")?;
    let cmd_result: ExitStatus = output.wait().context("command failed")?;
    tui.enter().context("failed to enter TUI mode again")?;
    if !cmd_result.success() {
        let error = format!(
            "Failed to execute command: {:?}, exit code: {}",
            cmd,
            cmd_result.code().unwrap_or(0),
        );
        log(error.as_str());
        return Err(anyhow!(error));
    }
    Ok(())
}

pub fn execute_shell(cmd: String) -> Result<()> {
    log(format!("Executing command: {:?}", cmd).as_str());
    let c = Command::new("sh")
        .arg("-c")
        .arg(cmd.clone())
        .stdin(Stdio::inherit())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to start a command")?;
    let output = c
        .wait_with_output()
        .context("failed to read command output")?;

    if !output.status.success() {
        let error = format!(
            "Failed to execute command: {:?}, {}\n{}\n{}",
            cmd,
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout),
        );
        log(error.as_str());
        return Err(anyhow!(error));
    }
    Ok(())
}

pub fn rename_file(abs_path: &String, new_name: &String) -> Result<()> {
    let path_parts = abs_path.split('/').collect::<Vec<&str>>();
    let folder_abs_path: String = path_parts[..path_parts.len() - 1].join("/");
    let cmd = format!("mv \"{}\" \"{}/{}\"", abs_path, folder_abs_path, new_name);
    execute_shell(cmd)
}

pub fn create_file(abs_path: &String) -> Result<()> {
    let cmd = format!("touch \"{}\"", abs_path);
    execute_shell(cmd)
}

pub fn create_directory(abs_path: &String) -> Result<()> {
    let cmd = format!("mkdir -p \"{}\"", abs_path);
    execute_shell(cmd)
}

pub fn delete_tree_node(tree_node: &TreeNode, abs_path: &String) -> Result<()> {
    let cmd: String = match &tree_node.kind {
        TreeNodeType::SelfReference => {
            format!("rm -rf \"{}\"", abs_path)
        }
        TreeNodeType::FileNode(file_node) => match file_node.file_type {
            FileType::Directory => {
                format!("rm -rf \"{}\"", abs_path)
            }
            _ => {
                format!("rm \"{}\"", abs_path)
            }
        },
    };
    execute_shell(cmd)
}

pub fn copy_path_to_clipboard(path: &String) -> Result<()> {
    let mut clipboard = Clipboard::new().context("failed to create clipboard")?;
    clipboard
        .set_text(path)
        .context("failed to copy path to clipboard")?;
    Ok(())
}

pub fn get_file_details(abs_path: &String, is_directory: bool) -> Result<String> {
    let file_type = match is_directory {
        true => "Directory",
        false => "File",
    };
    let mut info_message = format!("{}: {}", file_type, abs_path);

    let metadata = fs::metadata(abs_path).context("failed to read file metadata")?;
    let size_bytes = metadata.len();
    let file_size: String = human_readable_size(size_bytes);
    info_message.push_str(format!("\nSize: {}", file_size).as_str());

    let modified_time = metadata
        .modified()
        .context("failed to read modified time")?;
    let dt: DateTime<Utc> = modified_time.into();
    let modified_time_str = dt.format("%Y-%m-%d %H:%M:%S %z");
    info_message.push_str(format!("\nModified: {}", modified_time_str).as_str());

    Ok(info_message)
}

pub fn human_readable_size(size_bytes: u64) -> String {
    if size_bytes < 1024 {
        return format!("{} bytes", size_bytes);
    }
    let mut size = size_bytes as f64;
    let units = ["B", "kB", "MB", "GB", "TB"];
    let mut i = 0;
    while size >= 1000.0 && i < units.len() - 1 {
        size /= 1000.0;
        i += 1;
    }
    format!("{:.2} {} ({} bytes)", size, units[i], size_bytes)
}

pub fn run_custom_command(workdir: String, cmd: &String) -> Result<String> {
    log(format!("Executing command: {:?}", cmd).as_str());
    let c = Command::new("sh")
        .arg("-c")
        .arg(cmd.clone())
        .current_dir(workdir)
        .stdin(Stdio::inherit())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to start a command")?;
    let output = c
        .wait_with_output()
        .context("failed to read command output")?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let error = format!(
            "Failed to execute command: {:?}\nExit code: {}\n{}\n{}",
            cmd, output.status, stderr, stdout,
        );
        log(error.as_str());
        return Err(anyhow!(error));
    }
    Ok(format!(
        "Command \"{}\" executed successfully. Output:\n\n{}\n{}",
        cmd, stderr, stdout,
    ))
}
