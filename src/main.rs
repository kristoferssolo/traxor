use color_eyre::Result;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tracing::{debug, trace};
use traxor::{
    app::App,
    config::Config,
    event::{Event, EventHandler},
    handler::{get_action, update},
    telemetry::setup_logger,
    tui::Tui,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    debug!("Loading configuration...");
    let config = Config::load()?;
    debug!("Configuration loaded.");

    // Setup the logger.
    setup_logger(&config)?;

    // Create an application.
    let mut app = App::new(config)?;

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
            Event::Tick => {
                trace!(target: "app", "Event::Tick");
                app.tick().await?;
            }
            Event::Key(key_event) => {
                trace!(target: "app", "Event::Key: {:?}", key_event);
                if let Some(action) = get_action(key_event, &mut app).await? {
                    trace!(target: "app", "Action: {:?}", action);
                    update(&mut app, action).await?;
                }
            }
            Event::Mouse(mouse_event) => {
                trace!(target: "app", "Event::Mouse: {:?}", mouse_event);
            }
            Event::Resize(x, y) => {
                trace!(target: "app", "Event::Resize: ({}, {})", x, y);
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
