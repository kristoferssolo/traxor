use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use traxor::{app::action::Action, handler::get_action};

#[test]
fn test_get_action_quit() {
    assert_eq!(get_action(KeyEvent::from(KeyCode::Esc)), Some(Action::Quit));
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('q'))),
        Some(Action::Quit)
    );
    assert_eq!(
        get_action(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Some(Action::Quit)
    );
    assert_eq!(
        get_action(KeyEvent::new(KeyCode::Char('C'), KeyModifiers::CONTROL)),
        Some(Action::Quit)
    );
}

#[test]
fn test_get_action_navigation() {
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('l'))),
        Some(Action::NextTab)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Right)),
        Some(Action::NextTab)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('h'))),
        Some(Action::PrevTab)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Left)),
        Some(Action::PrevTab)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('j'))),
        Some(Action::NextTorrent)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Down)),
        Some(Action::NextTorrent)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('k'))),
        Some(Action::PrevTorrent)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Up)),
        Some(Action::PrevTorrent)
    );
}

#[test]
fn test_get_action_switch_tab() {
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('1'))),
        Some(Action::SwitchTab(0))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('2'))),
        Some(Action::SwitchTab(1))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('3'))),
        Some(Action::SwitchTab(2))
    );
}

#[test]
fn test_get_action_torrent_actions() {
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('t'))),
        Some(Action::ToggleTorrent)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Enter)),
        Some(Action::ToggleTorrent)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Menu)),
        Some(Action::ToggleTorrent)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('a'))),
        Some(Action::ToggleAll)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('d'))),
        Some(Action::Delete(false))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('D'))),
        Some(Action::Delete(true))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char(' '))),
        Some(Action::Select)
    );
}

#[test]
fn test_get_action_unhandled() {
    assert_eq!(get_action(KeyEvent::from(KeyCode::Char('x'))), None);
    assert_eq!(get_action(KeyEvent::from(KeyCode::F(1))), None);
}
