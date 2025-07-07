mod log;

use color_eyre::Result;
use log::setup_logger;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use traxor::{
    app::App,
    event::{Event, EventHandler},
    handler::{get_action, update},
    tui::Tui,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup the logger.
    setup_logger()?;

    // Create an application.
    let mut app = App::new()?;

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
            Event::Tick => app.tick().await?,
            // Event::Key(key_event) => handle_key_events(key_event, &mut app).await?,
            Event::Key(key_event) => {
                if let Some(action) = get_action(key_event) {
                    update(&mut app, action).await.unwrap();
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
