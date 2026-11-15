use color_eyre::Result;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io::{self, Stderr},
    sync::Arc,
};
use tokio::{
    sync::Mutex,
    time::{self, Duration},
};
use tracing::warn;
use traxor::{
    app::{
        App,
        constants::{DEFAULT_TICK_RATE_MS, TORRENT_UPDATE_INTERVAL_SECS},
    },
    config::Config,
    event::{Event, EventHandler},
    handler::{handle_key_event, handle_mouse},
    telemetry::setup_logger,
    tui::Tui,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config = Config::load()?;
    setup_logger(&config)?;

    let app = Arc::new(Mutex::new(App::new(config)?));
    spawn_torrent_updater(Arc::clone(&app));

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(DEFAULT_TICK_RATE_MS);
    let mut tui = Tui::new(terminal, events);

    let run_result = run(&mut tui, &app).await;
    let exit_result = tui.exit();
    run_result.and(exit_result)
}

/// Initializes the terminal and runs the event loop until the app quits.
///
/// The caller restores the terminal afterwards, whether this succeeds or fails.
async fn run(tui: &mut Tui<CrosstermBackend<Stderr>>, app: &Mutex<App>) -> Result<()> {
    tui.init()?;
    loop {
        let mut app_guard = app.lock().await;
        if !app_guard.running {
            return Ok(());
        }
        tui.draw(&mut app_guard)?;
        drop(app_guard);

        let event = tui.events.next()?;
        if let Event::Mouse(mouse_event) = event {
            handle_mouse(mouse_event, &mut *app.lock().await);
        }
        if let Event::Key(key_event) = event {
            let mut app_guard = app.lock().await;
            handle_key_event(key_event, &mut app_guard).await;
        }
    }
}

fn spawn_torrent_updater(app: Arc<Mutex<App>>) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(TORRENT_UPDATE_INTERVAL_SECS));
        loop {
            interval.tick().await;
            if let Err(e) = app.lock().await.torrents.update().await {
                warn!("Failed to update torrents: {e}");
            }
        }
    });
}
