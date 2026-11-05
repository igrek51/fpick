use anyhow::{Context, Result};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, panic};

pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;

use crate::{
    app::App,
    event::{Event, EventHandler},
    keyboard::update_on_key,
    ui,
};

#[derive(Debug)]
pub struct Tui {
    terminal: CrosstermTerminal,
    pub event_handler: EventHandler,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(std::io::stderr());
        let terminal: CrosstermTerminal = Terminal::new(backend).unwrap();
        let event_handler: EventHandler = EventHandler::new(500)?;
        Ok(Self {
            terminal,
            event_handler,
        })
    }

    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?; // https://docs.rs/crossterm/latest/crossterm/terminal/index.html#raw-mode
        crossterm::execute!(io::stderr(), EnterAlternateScreen)?;

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::fatal_exit().expect("failed to reset the terminal");
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
        if let Some(event) = self.event_handler.next()? {
            self.handle_event(app, event);
        }
        // Handle a burst of already queued events before redrawing, so fast
        // typing or key repeat costs one frame instead of one frame per key.
        while !app.should_quit {
            match self.event_handler.try_next()? {
                Some(event) => self.handle_event(app, event),
                None => break,
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, app: &mut App, event: Event) {
        match event {
            Event::Key(key_event) => update_on_key(app, key_event, self),
            Event::Resize => {}
        }
    }

    fn fatal_exit() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen)?;
        Ok(())
    }
}
