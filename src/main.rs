use color_eyre::Result;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stderr};
use tokio::{
    sync::mpsc,
    time::{self, Duration},
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

/// Fetches torrents on a background task and sends each outcome to the returned receiver.
///
/// The fetch never touches [`App`], so a slow daemon cannot stall drawing or input.
/// The task ends once the receiver is dropped.
fn spawn_torrent_updater(mut fetcher: TorrentFetcher) -> mpsc::Receiver<Refresh> {
    let (sender, receiver) = mpsc::channel(1);
    let pause = Duration::from_secs(TORRENT_UPDATE_INTERVAL_SECS);
    tokio::spawn(repeat_with_pause(
        async move || fetcher.fetch().await,
        sender,
        pause,
    ));
    receiver
}

/// Sends `fetch` results to `sender` until it closes, waiting `pause` after each send.
///
/// The pause starts when a fetch finishes, so a fetch slower than `pause` is not followed
/// immediately by another one, as an overdue interval tick would do.
async fn repeat_with_pause<T>(
    mut fetch: impl AsyncFnMut() -> T,
    sender: mpsc::Sender<T>,
    pause: Duration,
) {
    while sender.send(fetch().await).await.is_ok() {
        time::sleep(pause).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_ok, assert_some};
    use std::time::Instant;

    #[tokio::test]
    async fn slow_fetches_are_followed_by_a_full_pause() {
        let pause = Duration::from_millis(50);
        let fetch_time = pause * 2;
        let (sender, mut receiver) = mpsc::channel(1);
        tokio::spawn(repeat_with_pause(
            async move || {
                let started = Instant::now();
                time::sleep(fetch_time).await;
                (started, Instant::now())
            },
            sender,
            pause,
        ));

        let (_, first_finished) = assert_some!(receiver.recv().await);
        let (second_started, _) = assert_some!(receiver.recv().await);

        assert!(second_started.duration_since(first_finished) >= pause);
    }

    #[tokio::test]
    async fn stops_once_the_receiver_is_dropped() {
        let (sender, receiver) = mpsc::channel(1);
        drop(receiver);

        let task = tokio::spawn(repeat_with_pause(async || {}, sender, Duration::ZERO));

        assert_ok!(assert_ok!(
            time::timeout(Duration::from_secs(5), task).await
        ));
    }
}
