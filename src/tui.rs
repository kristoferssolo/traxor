use crate::app::App;
use crate::event::EventHandler;
use crate::ui;
use color_eyre::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::Backend;
use std::io;
use std::panic;
use tracing::error;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui<B: Backend> {
    /// Interface to the Terminal.
    terminal: Terminal<B>,
    /// Terminal event handler.
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    /// Constructs a new instance of [`Tui`].
    pub const fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }
}

impl<B: Backend<Error: Send + Sync + 'static>> Tui<B> {
    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal initialization fails.
    pub fn init(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            if let Err(e) = Self::reset() {
                let msg = format!("Error resetting terminal: {e:?}");
                eprintln!("{msg}");
                error!(msg);
            }
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: crate::tui::Terminal::draw
    /// [`rendering`]: crate::ui::render
    ///
    /// # Errors
    ///
    /// Returns an error if drawing fails.
    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| ui::render(app, frame))?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal reset fails.
    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal cleanup fails.
    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
