mod action_menu;
mod app;
mod app_logic;
mod appdata;
mod background;
mod errors;
mod event;
mod filesystem;
mod keyboard;
mod logs;
mod numbers;
mod tree;
mod tui;
mod ui;

use std::process::ExitCode;

use anyhow::Context;

use crate::app::App;

fn main() -> ExitCode {
    let mut app = App::new();
    app.run().context("app failed").unwrap();
    match app.exit_code {
        0 => ExitCode::SUCCESS,
        _ => ExitCode::from(app.exit_code as u8),
    }
}
