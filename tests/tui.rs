use ratatui::backend::TestBackend;
use ratatui::Terminal;
use traxor::event::EventHandler;
use traxor::tui::Tui;

#[test]
fn test_tui_new() {
    let backend = TestBackend::new(10, 10);
    let terminal = Terminal::new(backend).unwrap();
    let events = EventHandler::new(250); // Dummy tick_rate
    let _tui = Tui::new(terminal, events);
    // Add assertions for initial state of Tui if applicable
    // For example, if Tui has a field that should be initialized to a specific value
}
