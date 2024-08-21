use anyhow::Result;
use std::process::{Command, Stdio};

use crate::logs::log;

#[derive(Debug, Clone)]
pub struct MenuAction {
    pub name: &'static str,
    pub operation: Operation,
}

#[derive(Debug, Clone)]
pub enum Operation {
    ShellCommand { template: &'static str },
    PickAbsolutePath,
    PickRelativePath,
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
            operation: Operation::ShellCommand {
                template: "gnome-terminal -- less \"{}\"",
            },
        },
        MenuAction {
            name: "Edit in vim",
            operation: Operation::ShellCommand {
                template: "gnome-terminal -- vim \"{}\"",
            },
        },
        MenuAction {
            name: "Edit in sudo vim",
            operation: Operation::ShellCommand {
                template: "gnome-terminal -- sudo vim \"{}\"",
            },
        },
        MenuAction {
            name: "Delete file",
            operation: Operation::ShellCommand {
                template: "rm \"{}\"",
            },
        },
        MenuAction {
            name: "Delete directory",
            operation: Operation::ShellCommand {
                template: "rm -rf \"{}\"",
            },
        },
        MenuAction {
            name: "Copy filename to clipboard",
            operation: Operation::ShellCommand {
                template: "echo -n \"{}\" | xclip -selection clipboard",
            },
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
    log(format!("Executing command: {:?}", cmd).as_str());
    let mut c = Command::new("sh")
        .arg("-c")
        .arg(cmd.clone())
        .stdin(Stdio::inherit())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    c.wait()?;
    Ok(())
}
