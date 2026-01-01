use color_eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
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
#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    ///
    /// # Panics
    ///
    /// Panics if event polling or sending fails.
    #[must_use]
    pub fn new(tick_rate_ms: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate_ms);
        let (sender, receiver) = mpsc::channel();

        let handler = thread::spawn(move || {
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

        Self { receiver, handler }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    ///
    /// # Errors
    ///
    /// Returns an error if the sender is disconnected.
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}
