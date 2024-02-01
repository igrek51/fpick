mod app;
mod appdata;
mod event;
mod filesystem;
mod numbers;
mod tui;
mod ui;
mod update;

use anyhow::Result;

use crate::app::App;

fn main() -> Result<()> {
    let mut app = App::new();
    app.run()?;
    Ok(())
}
