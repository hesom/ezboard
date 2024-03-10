use std::panic;

use anyhow::Result;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::Backend, Terminal};

use crate::{app::App, event::EventStream, ui};

pub struct Tui<B: Backend> {
    /// Interface to the terminal.
    terminal: Terminal<B>,
    /// Event stream is owned by terminal
    pub event_stream: EventStream,
}

impl<B: Backend> Tui<B> {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: Terminal<B>, event_stream: EventStream) -> Self {
        Self {
            terminal,
            event_stream,
        }
    }

    /// Initialized the terminal interface
    ///
    /// It enables raw mode and sets terminal properties.
    pub fn init(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), EnterAlternateScreen)?;

        // Define a custom panic hook to reset the terminal properties
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: ratatui::Terminal::draw
    /// ['rendering`]: crate::ui::render
    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| ui::render(app, frame))?;
        Ok(())
    }

    /// Resets the terminal interface
    ///
    /// This function is also used for the panic hook to revert
    /// tthe terminal properties if unexpected errors occur.
    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    /// Exits the terminal interface
    ///
    /// It disables raw mode and reverts back the terminal
    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        Ok(())
    }
}
