use color_eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
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
#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    /// Event handler thread.
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    ///
    /// # Panics
    ///
    /// TODO: add panic
    #[must_use]
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("no events available") {
                        match event::read() {
                            Ok(CrosstermEvent::Key(e)) => sender.send(Event::Key(e)),
                            Ok(CrosstermEvent::Mouse(e)) => sender.send(Event::Mouse(e)),
                            Ok(CrosstermEvent::Resize(w, h)) => sender.send(Event::Resize(w, h)),
                            Err(e) => {
                                error!("Error reading event: {:?}", e);
                                break;
                            }
                            _ => Ok(()), // Ignore other events
                        }
                        .expect("failed to send terminal event");
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender.send(Event::Tick).expect("failed to send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    ///
    /// # Errors
    ///
    /// TODO: add error types
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}
