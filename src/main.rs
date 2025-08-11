use color_eyre::Result;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, sync::Arc};
use tokio::{
    sync::Mutex,
    time::{self, Duration},
};
use tracing::{trace, warn};
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

    let config = Config::load()?;
    setup_logger(&config)?;

    // Wrap App in Arc<Mutex<>> so we can share it between UI and updater
    let app = Arc::new(Mutex::new(App::new(config)?));

    // Clone for updater task
    let app_clone = app.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;

            let mut app = app_clone.lock().await;
            if let Err(e) = app.torrents.update().await {
                warn!("Failed to update torrents: {e}");
            }
        }
    });

    // TUI setup
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250); // Update time in ms
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Main loop
    loop {
        {
            let app_guard = app.lock().await;
            if !app_guard.running {
                break;
            }
        }

        {
            let mut app_guard = app.lock().await;
            tui.draw(&mut app_guard)?;
        }

        match tui.events.next()? {
            Event::Tick => {}
            Event::Key(key_event) => {
                let mut app_guard = app.lock().await;
                if let Some(action) = get_action(key_event, &mut app_guard).await? {
                    update(&mut app_guard, action).await?;
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

    tui.exit()?;
    Ok(())
}
