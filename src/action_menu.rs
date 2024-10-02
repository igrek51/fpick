use anyhow::{anyhow, Context, Result};
use arboard::Clipboard;
use std::process::{Command, ExitStatus, Stdio};

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
