use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use traxor::app::App;
use traxor::event::{Event, EventHandler};
use traxor::handler::handle_key_events;
use traxor::tui::Tui;

#[tokio::main]
async fn main() -> Result<()> {
    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250); // Update time in ms
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick().await,
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
