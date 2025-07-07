use crossterm::event::{KeyCode, KeyEvent};
use traxor::{app::action::Action, handler::get_action, app::App, config::Config};

#[test]
fn test_get_action_quit() {
    let config = Config::load().unwrap();
    let app = App::new(config).unwrap();
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('q')), &app),
        Some(Action::Quit)
    );
}

#[test]
fn test_get_action_navigation() {
    let config = Config::load().unwrap();
    let app = App::new(config).unwrap();
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('l')), &app),
        Some(Action::NextTab)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('h')), &app),
        Some(Action::PrevTab)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('j')), &app),
        Some(Action::NextTorrent)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('k')), &app),
        Some(Action::PrevTorrent)
    );
}

#[test]
fn test_get_action_switch_tab() {
    let config = Config::load().unwrap();
    let app = App::new(config).unwrap();
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('1')), &app),
        Some(Action::SwitchTab(0))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('2')), &app),
        Some(Action::SwitchTab(1))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('3')), &app),
        Some(Action::SwitchTab(2))
    );
}

#[test]
fn test_get_action_torrent_actions() {
    let config = Config::load().unwrap();
    let app = App::new(config).unwrap();
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Enter), &app),
        Some(Action::ToggleTorrent)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('a')), &app),
        Some(Action::ToggleAll)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('d')), &app),
        Some(Action::Delete(false))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('D')), &app),
        Some(Action::Delete(true))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char(' ')), &app),
        Some(Action::Select)
    );
}

#[test]
fn test_get_action_unhandled() {
    let config = Config::load().unwrap();
    let app = App::new(config).unwrap();
    assert_eq!(get_action(KeyEvent::from(KeyCode::Char('x')), &app), None);
    assert_eq!(get_action(KeyEvent::from(KeyCode::F(1)), &app), None);
}
