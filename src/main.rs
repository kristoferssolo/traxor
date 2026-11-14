use color_eyre::Result;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stderr};
use tokio::{
    sync::mpsc,
    time::{self, Duration, MissedTickBehavior},
};
use transmission_rpc::types::Torrent;
use traxor::{
    app::{
        App, TorrentFetcher,
        constants::{DEFAULT_TICK_RATE_MS, TORRENT_UPDATE_INTERVAL_SECS},
    },
    config::Config,
    event::EventHandler,
    handler::handle_event,
    telemetry::setup_logger,
    tui::Tui,
};

/// Outcome of one background torrent list fetch.
type Refresh = traxor::error::Result<Vec<Torrent>>;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config = Config::load()?;
    setup_logger(&config)?;

    let mut app = App::new(config)?;
    let refreshes = spawn_torrent_updater(app.torrents.fetcher(&app.config.rpc)?);

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(DEFAULT_TICK_RATE_MS);
    let mut tui = Tui::new(terminal, events);

    let run_result = run(&mut tui, &mut app, refreshes).await;
    let exit_result = tui.exit();
    run_result.and(exit_result)
}

/// Initializes the terminal and runs the event loop until the app quits.
///
/// Redraws after every terminal event and every torrent refresh.
/// The caller restores the terminal afterwards, whether this succeeds or fails.
async fn run(
    tui: &mut Tui<CrosstermBackend<Stderr>>,
    app: &mut App,
    mut refreshes: mpsc::Receiver<Refresh>,
) -> Result<()> {
    tui.init()?;
    while app.running {
        tui.draw(app)?;
        tokio::select! {
            event = tui.events.next() => handle_event(event?, app).await,
            Some(refresh) = refreshes.recv() => app.apply_refresh(refresh),
        }
    }
    Ok(())
}

/// Fetches torrents every [`TORRENT_UPDATE_INTERVAL_SECS`] on a background task and
/// sends each outcome to the returned receiver.
///
/// The fetch never touches [`App`], so a slow daemon cannot stall drawing or input.
/// The task ends once the receiver is dropped.
fn spawn_torrent_updater(mut fetcher: TorrentFetcher) -> mpsc::Receiver<Refresh> {
    let (sender, receiver) = mpsc::channel(1);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(TORRENT_UPDATE_INTERVAL_SECS));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            if sender.send(fetcher.fetch().await).await.is_err() {
                break;
            }
        }
    });
    receiver
}
