use crossterm::event::{KeyCode, KeyEvent};
use traxor::{app::App, app::InputMode, app::action::Action, config::Config, handler::get_action};

#[tokio::test]
async fn test_get_action_quit() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('q')), &mut app)
            .await
            .unwrap(),
        Some(Action::Quit)
    );
}

#[tokio::test]
async fn test_get_action_navigation() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('l')), &mut app)
            .await
            .unwrap(),
        Some(Action::NextTab)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('h')), &mut app)
            .await
            .unwrap(),
        Some(Action::PrevTab)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('j')), &mut app)
            .await
            .unwrap(),
        Some(Action::NextTorrent)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('k')), &mut app)
            .await
            .unwrap(),
        Some(Action::PrevTorrent)
    );
}

#[tokio::test]
async fn test_get_action_switch_tab() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('1')), &mut app)
            .await
            .unwrap(),
        Some(Action::SwitchTab(0))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('2')), &mut app)
            .await
            .unwrap(),
        Some(Action::SwitchTab(1))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('3')), &mut app)
            .await
            .unwrap(),
        Some(Action::SwitchTab(2))
    );
}

#[tokio::test]
async fn test_get_action_torrent_actions() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Enter), &mut app)
            .await
            .unwrap(),
        Some(Action::ToggleTorrent)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('a')), &mut app)
            .await
            .unwrap(),
        Some(Action::ToggleAll)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('d')), &mut app)
            .await
            .unwrap(),
        Some(Action::Delete(false))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('D')), &mut app)
            .await
            .unwrap(),
        Some(Action::Delete(true))
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char(' ')), &mut app)
            .await
            .unwrap(),
        Some(Action::Select)
    );
}

#[tokio::test]
async fn test_get_action_unhandled() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('x')), &mut app)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::F(1)), &mut app)
            .await
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn test_get_action_toggle_help() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Char('?')), &mut app)
            .await
            .unwrap(),
        Some(Action::ToggleHelp)
    );
}

#[tokio::test]
async fn test_get_action_input_mode() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    app.input_mode = InputMode::Move;
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Enter), &mut app)
            .await
            .unwrap(),
        Some(Action::Submit)
    );
    assert_eq!(
        get_action(KeyEvent::from(KeyCode::Esc), &mut app)
            .await
            .unwrap(),
        Some(Action::Cancel)
    );
}
