use traxor::event::Event;

#[test]
fn test_event_from_key_code() {
    // Test cases for various key codes
    // Add more as needed based on your Event enum definition
    let key_event = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('q'),
        crossterm::event::KeyModifiers::NONE,
    );
    assert_eq!(Event::Key(key_event), Event::Key(key_event));
}
