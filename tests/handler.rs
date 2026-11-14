use claims::{assert_none, assert_ok, assert_some, assert_some_eq};
use crossterm::event::{KeyCode, KeyEvent};
use std::{fs, time::Duration};
use tempfile::TempDir;
use tokio::{net::TcpListener, time::timeout};
use transmission_rpc::types::Torrent;
use traxor::{
    app::App,
    app::InputMode,
    app::action::Action,
    config::Config,
    handler::{get_action, handle_key_event},
};

macro_rules! assert_action {
    ($app:expr, $key_code:expr) => {
        assert_ok!(get_action(KeyEvent::from($key_code), $app).await)
    };
}

#[tokio::test]
async fn get_action_quit() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_some_eq!(assert_action!(&mut app, KeyCode::Char('q')), Action::Quit);
}

#[tokio::test]
async fn get_action_navigation() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('l')),
        Action::NextTab
    );
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('h')),
        Action::PrevTab
    );
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('j')),
        Action::NextTorrent
    );
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('k')),
        Action::PrevTorrent
    );
}

#[tokio::test]
async fn get_action_switch_tab() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('1')),
        Action::SwitchTab(0)
    );
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('2')),
        Action::SwitchTab(1)
    );
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('3')),
        Action::SwitchTab(2)
    );
}

#[tokio::test]
async fn get_action_torrent_actions() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Enter),
        Action::ToggleTorrent
    );
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('a')),
        Action::ToggleAll
    );
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('d')),
        Action::Delete(false)
    );
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('D')),
        Action::Delete(true)
    );
    assert_some_eq!(assert_action!(&mut app, KeyCode::Char(' ')), Action::Select);
}

#[tokio::test]
async fn get_action_unhandled() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_none!(assert_action!(&mut app, KeyCode::Char('x')));
    assert_none!(assert_action!(&mut app, KeyCode::F(1)));
}

#[tokio::test]
async fn get_action_toggle_help() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('?')),
        Action::ToggleHelp
    );
}

#[tokio::test]
async fn get_action_input_mode() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.input_mode = InputMode::Move;
    assert_some_eq!(assert_action!(&mut app, KeyCode::Enter), Action::Submit);
    assert_some_eq!(assert_action!(&mut app, KeyCode::Esc), Action::Cancel);
}

#[tokio::test]
async fn get_action_confirm_delete() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.input_mode = InputMode::ConfirmDelete(false);

    for key_code in [KeyCode::Enter, KeyCode::Char('y'), KeyCode::Char('Y')] {
        assert_some_eq!(assert_action!(&mut app, key_code), Action::ConfirmYes);
    }

    for key_code in [KeyCode::Char('n'), KeyCode::Char('N'), KeyCode::Esc] {
        assert_some_eq!(assert_action!(&mut app, key_code), Action::Cancel);
    }

    assert_none!(assert_action!(&mut app, KeyCode::Char('x')));
}

#[tokio::test]
async fn get_action_tab_completes_move_input() {
    let dir = temp_dir();
    assert_ok!(fs::create_dir(dir.path().join("alpha")));
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.input_mode = InputMode::Move;
    app.input_handler
        .set_text(format!("{}/a", dir.path().display()));

    assert_none!(assert_action!(&mut app, KeyCode::Tab));
    assert_eq!(
        app.input_handler.text,
        format!("{}/alpha/", dir.path().display())
    );
}

#[tokio::test]
async fn get_action_tab_does_not_complete_rename_or_filter_input() {
    let dir = temp_dir();
    assert_ok!(fs::create_dir(dir.path().join("alpha")));

    for input_mode in [InputMode::Rename, InputMode::Filter] {
        let config = assert_ok!(Config::load());
        let mut app = assert_ok!(App::new(config));
        let input = format!("{}/a", dir.path().display());
        app.input_mode = input_mode;
        app.input_handler.set_text(input.clone());

        assert_none!(assert_action!(&mut app, KeyCode::Tab));
        assert_eq!(app.input_handler.text, input);
    }
}

#[tokio::test]
async fn handle_key_event_reports_failed_action_and_keeps_running() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_ok!(app.torrents.url(&failing_rpc_url().await));
    app.torrents.torrents = vec![assert_ok!(toml::from_str::<Torrent>(
        "id = 1\nname = \"old\""
    ))];
    app.state.select(Some(0));
    app.prepare_rename_action();
    app.input_handler.set_text("new".to_owned());

    assert_ok!(
        timeout(
            Duration::from_secs(10),
            handle_key_event(KeyEvent::from(KeyCode::Enter), &mut app),
        )
        .await
    );

    assert!(app.running);
    assert_eq!(app.input_mode, InputMode::Rename);
    assert_eq!(app.input_handler.text, "new");
    let message = assert_some!(app.status_message.as_ref());
    assert!(message.text.starts_with("Transmission RPC error"));

    handle_key_event(KeyEvent::from(KeyCode::Esc), &mut app).await;

    assert_none!(app.status_message);
    assert_eq!(app.input_mode, InputMode::None);
}

/// Starts a local server that closes every connection without answering, so
/// RPC requests to the returned URL fail.
///
/// The server task keeps the port bound until the test's runtime shuts down,
/// so no other process can take the port and answer instead.
async fn failing_rpc_url() -> String {
    let listener = assert_ok!(TcpListener::bind("127.0.0.1:0").await);
    let port = assert_ok!(listener.local_addr()).port();
    assert_ne!(port, 9091, "must not reach a real Transmission daemon");
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            drop(stream);
        }
    });
    format!("http://127.0.0.1:{port}/transmission/rpc")
}

fn temp_dir() -> TempDir {
    TempDir::new().expect("temp directory")
}
