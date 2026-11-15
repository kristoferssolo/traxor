use claims::{assert_none, assert_ok, assert_some, assert_some_eq};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{Terminal, backend::TestBackend, layout::Rect, text::Line};
use std::{fs, time::Duration};
use tempfile::TempDir;
use tokio::{net::TcpListener, time::timeout};
use transmission_rpc::types::Torrent;
use traxor::{
    app::App,
    app::InputMode,
    app::action::Action,
    app::types::DeleteTarget,
    config::Config,
    handler::{get_action, handle_key_event, handle_mouse, update},
    ui::render,
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
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('g')),
        Action::FirstTorrent
    );
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('G')),
        Action::LastTorrent
    );
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::PageDown),
        Action::PageDown
    );
    assert_some_eq!(assert_action!(&mut app, KeyCode::PageUp), Action::PageUp);
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
    assert_some_eq!(assert_action!(&mut app, ctrl('a')), Action::SelectAll);
    assert_some_eq!(assert_action!(&mut app, ctrl('u')), Action::ClearSelection);
}

#[tokio::test]
async fn get_action_sort() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_some_eq!(
        assert_action!(&mut app, KeyCode::Char('s')),
        Action::SortColumn
    );
    assert_some_eq!(
        assert_ok!(
            get_action(
                KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT),
                &mut app
            )
            .await
        ),
        Action::SortReverse
    );
}

#[tokio::test]
async fn update_sort_actions_change_current_tab_sort() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));

    assert_ok!(update(&mut app, Action::SortColumn).await);
    let sort = assert_some!(app.tabs()[0].sort());
    assert_ok!(update(&mut app, Action::SortReverse).await);
    assert_some_eq!(app.tabs()[0].sort(), sort.reversed());
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
    let target = assert_some!(DeleteTarget::new(vec![(1, "alpha".into())], false));
    app.input_mode = InputMode::ConfirmDelete(target);

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

#[test]
fn handle_mouse_click_switches_tab() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.areas.tabs = Rect::new(0, 0, 200, 3);

    handle_mouse(click_on_second_tab(&app), &mut app);
    assert_eq!(app.index(), 1);
}

#[test]
fn handle_mouse_ignored_while_dialog_open() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.areas.tabs = Rect::new(0, 0, 200, 3);
    let click = click_on_second_tab(&app);

    app.show_help = true;
    handle_mouse(click, &mut app);
    assert_eq!(app.index(), 0);

    app.show_help = false;
    app.input_mode = InputMode::Filter;
    handle_mouse(click, &mut app);
    assert_eq!(app.index(), 0);
}

#[test]
fn handle_mouse_click_highlights_rendered_torrent_after_refresh() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![
        torrent(1, "alpha"),
        torrent(2, "bravo"),
        torrent(3, "charlie"),
    ];
    let mut terminal = assert_ok!(Terminal::new(TestBackend::new(120, 20)));
    assert_ok!(terminal.draw(|frame| render(&mut app, frame)));

    // A background refresh removes the first rendered torrent before the click arrives.
    app.torrents.torrents.remove(0);
    let column = app.areas.table.x + 2;
    // Skip the top border, header, and header margin.
    let first_row = app.areas.table.y + 3;

    handle_mouse(left_click(column, first_row), &mut app);
    assert_none!(app.state.selected());

    handle_mouse(left_click(column, first_row + 1), &mut app);
    let highlighted = assert_some!(app.state.selected());
    assert_some_eq!(app.filtered_torrents()[highlighted].id, 2);
}

#[test]
fn handle_mouse_click_moves_existing_highlight_across_redraws() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![
        torrent(1, "alpha"),
        torrent(2, "bravo"),
        torrent(3, "charlie"),
    ];
    app.next();
    let mut terminal = assert_ok!(Terminal::new(TestBackend::new(120, 20)));
    assert_ok!(terminal.draw(|frame| render(&mut app, frame)));
    let column = app.areas.table.x + 2;
    // Skip the top border, header, and header margin.
    let third_row = app.areas.table.y + 3 + 2;

    handle_mouse(left_click(column, third_row), &mut app);
    // Each draw re-syncs the table row from the highlighted torrent, so the click must move it.
    assert_ok!(terminal.draw(|frame| render(&mut app, frame)));

    let highlighted = assert_some!(app.state.selected());
    assert_some_eq!(app.filtered_torrents()[highlighted].id, 3);
}

/// Left click on the first title character of the second tab: border, padded first title,
/// divider, then left padding.
fn click_on_second_tab(app: &App) -> MouseEvent {
    let first_title_width = assert_ok!(u16::try_from(Line::from(app.tabs()[0].name()).width()));
    left_click(1 + first_title_width + 2 + 1 + 1, 1)
}

const fn left_click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

/// Builds a torrent with only `id` and `name` set; every other field deserializes to `None`.
fn torrent(id: i64, name: &str) -> Torrent {
    assert_ok!(toml::from_str(&format!("id = {id}\nname = {name:?}")))
}

#[tokio::test]
async fn handle_key_event_reports_failed_action_and_keeps_running() {
    let mut config = assert_ok!(Config::load());
    config.rpc.url = failing_rpc_url().await;
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![assert_ok!(toml::from_str::<Torrent>(
        "id = 1\nname = \"old\""
    ))];
    app.next();
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

#[tokio::test]
async fn get_action_modified_chars_edit_instead_of_inserting() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.input_mode = InputMode::Rename;
    app.input_handler.set_text("/downloads/shows".to_owned());

    for (ch, modifiers) in [
        ('w', KeyModifiers::CONTROL),
        ('x', KeyModifiers::ALT),
        ('S', KeyModifiers::SHIFT),
    ] {
        assert_none!(assert_ok!(
            get_action(KeyEvent::new(KeyCode::Char(ch), modifiers), &mut app).await
        ));
    }

    assert_eq!(app.input_handler.text, "/downloads/S");
}

#[tokio::test]
async fn get_action_ctrl_alt_chars_type_altgr_text_only_on_windows() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.input_mode = InputMode::Rename;
    app.input_handler.set_text("/downloads/shows".to_owned());
    let altgr = KeyModifiers::CONTROL | KeyModifiers::ALT;

    for ch in ['@', 'a', 'e', 'w', 'u'] {
        assert_none!(assert_ok!(
            get_action(KeyEvent::new(KeyCode::Char(ch), altgr), &mut app).await
        ));
    }

    // No Ctrl+Alt char triggers an editing shortcut; only Windows treats them as AltGr text.
    let expected = if cfg!(windows) {
        "/downloads/shows@aewu"
    } else {
        "/downloads/shows"
    };
    assert_eq!(app.input_handler.text, expected);
    assert_eq!(
        app.input_handler.cursor_position,
        app.input_handler.text.len()
    );
}

const fn ctrl(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL)
}

fn temp_dir() -> TempDir {
    TempDir::new().expect("temp directory")
}
