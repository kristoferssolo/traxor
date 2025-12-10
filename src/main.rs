use color_eyre::Result;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, sync::Arc};
use tokio::{
    sync::Mutex,
    task::JoinHandle,
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
    handler::{get_action, update},
    telemetry::setup_logger,
    tui::Tui,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config = Config::load()?;
    setup_logger(&config)?;

    let app = Arc::new(Mutex::new(App::new(config)?));

    spawn_torrent_updater(app.clone());

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(DEFAULT_TICK_RATE_MS);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

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
            Event::Key(key_event) => {
                let mut app_guard = app.lock().await;
                if let Some(action) = get_action(key_event, &mut app_guard).await? {
                    update(&mut app_guard, action).await?;
                }
            }
            Event::Mouse(_) | Event::Resize(_, _) | Event::Tick => {}
        }
    }

    tui.exit()?;
    Ok(())
}

fn spawn_torrent_updater(app: Arc<Mutex<App>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(TORRENT_UPDATE_INTERVAL_SECS));
        loop {
            interval.tick().await;
            let mut app = app.lock().await;
            if let Err(e) = app.torrents.update().await {
                warn!("Failed to update torrents: {e}");
            }
        }
    })
}
