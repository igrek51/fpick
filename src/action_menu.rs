use anyhow::Result;
use std::process::{Command, Stdio};

use crate::logs::log;

#[derive(Debug, Clone)]
pub struct MenuAction {
    pub name: &'static str,
    pub command: &'static str,
}

pub fn generate_known_actions() -> Vec<MenuAction> {
    vec![
        MenuAction {
            name: "Edit in vim",
            command: "gnome-terminal -- vim \"{}\"",
        },
        MenuAction {
            name: "Open in less",
            command: "gnome-terminal -- less \"{}\"",
        },
        MenuAction {
            name: "Delete file",
            command: "rm \"{}\"",
        },
        MenuAction {
            name: "Delete directory",
            command: "rm -rf \"{}\"",
        },
        MenuAction {
            name: "Copy filename to clipboard",
            command: "echo \"{}\" | xclip",
        },
    ]
}

pub fn run_menu_action(path: &String, menu_action: &MenuAction) -> Result<()> {
    let command = menu_action.command;
    let cmd = String::from(command).replace("{}", path);
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
