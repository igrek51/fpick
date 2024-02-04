mod app;
mod app_logic;
mod event;
mod filesystem;
mod numbers;
mod tree;
mod tui;
mod ui;
mod update;

use anyhow::Result;

use crate::app::App;

fn main() -> Result<()> {
    let mut app = App::new();
    app.run()?;
    if app.exit_code != 0 {
        std::process::exit(app.exit_code);
    }
    Ok(())
}
