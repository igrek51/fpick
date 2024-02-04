use anyhow::{Context, Result};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, panic};

pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;

use crate::{app::App, event::Event, event::EventHandler, ui, update::update};

pub struct Tui {
    terminal: CrosstermTerminal,
    pub events: EventHandler,
}

impl Tui {
    pub fn new() -> Self {
        let backend = CrosstermBackend::new(std::io::stderr());
        let terminal: CrosstermTerminal = Terminal::new(backend).unwrap();
        let events: EventHandler = EventHandler::new(5000);
        Self { terminal, events }
    }

    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen,)?;

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal
            .draw(|frame| ui::render(app, frame))
            .context("terminal.draw")?;
        Ok(())
    }

    pub fn handle_events(&mut self, app: &mut App) -> Result<()> {
        match self.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => update(app, key_event),
            Event::Resize(_, _) => {}
        };
        Ok(())
    }

    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen,)?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        Ok(())
    }
}
