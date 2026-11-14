use color_eyre::{Result, eyre::eyre};
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::{
    thread,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tracing::error;

/// Terminal events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

/// Terminal event handler.
///
/// A dedicated thread polls the terminal and forwards events over a channel, so
/// async code can await them with [`Self::next`].
#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Spawns a thread that forwards terminal events and sends [`Event::Tick`]
    /// every `tick_rate_ms`.
    ///
    /// # Panics
    ///
    /// Panics if the OS cannot create the event thread. The spawned thread panics if
    /// event polling fails.
    #[must_use]
    pub fn new(tick_rate_ms: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate_ms);
        let (sender, receiver) = mpsc::unbounded_channel();

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate.saturating_sub(last_tick.elapsed());

                if event::poll(timeout).expect("event polling failed") {
                    let send_result = match event::read() {
                        Ok(CrosstermEvent::Key(e)) => sender.send(Event::Key(e)),
                        Ok(CrosstermEvent::Mouse(e)) => sender.send(Event::Mouse(e)),
                        Ok(CrosstermEvent::Resize(w, h)) => sender.send(Event::Resize(w, h)),
                        Ok(_) => Ok(()),
                        Err(e) => {
                            error!("Error reading event: {e:?}");
                            break;
                        }
                    };
                    if send_result.is_err() {
                        break;
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if sender.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        Self { receiver }
    }

    /// Waits for the next event from the handler thread.
    ///
    /// Waiting yields to the async runtime instead of blocking its worker thread.
    ///
    /// # Errors
    ///
    /// Returns an error once the handler thread has stopped and every queued event
    /// has been received.
    pub async fn next(&mut self) -> Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_else(|| eyre!("terminal event thread stopped"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok, assert_ok_eq};
    use crossterm::event::{KeyCode, KeyModifiers};
    use tokio::time::timeout;

    const ENTER: Event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    /// `#[tokio::test]` runs on one thread, so a blocking receive would never let the
    /// sending task run.
    #[tokio::test]
    async fn next_waits_without_blocking_the_runtime() {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut events = EventHandler { receiver };
        let sending = tokio::spawn(async move { sender.send(ENTER) });

        let event = assert_ok!(timeout(Duration::from_secs(5), events.next()).await);

        assert_ok_eq!(event, ENTER);
        assert_ok!(assert_ok!(sending.await));
    }

    #[tokio::test]
    async fn next_drains_queued_events_before_failing_on_disconnect() {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut events = EventHandler { receiver };
        assert_ok!(sender.send(ENTER));
        assert_ok!(sender.send(Event::Tick));
        drop(sender);

        assert_ok_eq!(events.next().await, ENTER);
        assert_ok_eq!(events.next().await, Event::Tick);
        let error = assert_err!(events.next().await);
        assert_eq!(error.to_string(), "terminal event thread stopped");
    }
}
