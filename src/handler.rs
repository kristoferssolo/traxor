use crate::app::{action::Action, App};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tracing::{event, info_span, Level};

async fn handle_input(key_event: KeyEvent, app: &mut App<'_>) -> Result<Option<Action>> {
    match key_event.code {
        KeyCode::Enter => Ok(Some(Action::Submit)),
        KeyCode::Tab => {
            app.complete_input().await?;
            Ok(None)
        }
        KeyCode::Char(c) => {
            app.input.push(c);
            app.cursor_position = app.input.len();
            Ok(None)
        }
        KeyCode::Backspace => {
            app.input.pop();
            app.cursor_position = app.input.len();
            Ok(None)
        }
        KeyCode::Esc => Ok(Some(Action::Cancel)),
        _ => Ok(None),
    }
}

/// Handles the key events of [`App`].
#[tracing::instrument]
pub async fn get_action(key_event: KeyEvent, app: &mut App<'_>) -> Result<Option<Action>> {
    if app.input_mode {
        return handle_input(key_event, app).await;
    }

    let span = info_span!("get_action");
    let _enter = span.enter();
    event!(Level::INFO, "handling key event: {:?}", key_event);

    let keybinds = &app.config.keybinds;

    let actions = [
        (Action::Quit, &keybinds.quit),
        (Action::NextTab, &keybinds.next_tab),
        (Action::PrevTab, &keybinds.prev_tab),
        (Action::NextTorrent, &keybinds.next_torrent),
        (Action::PrevTorrent, &keybinds.prev_torrent),
        (Action::SwitchTab(0), &keybinds.switch_tab_1),
        (Action::SwitchTab(1), &keybinds.switch_tab_2),
        (Action::SwitchTab(2), &keybinds.switch_tab_3),
        (Action::ToggleTorrent, &keybinds.toggle_torrent),
        (Action::ToggleAll, &keybinds.toggle_all),
        (Action::Delete(false), &keybinds.delete),
        (Action::Delete(true), &keybinds.delete_force),
        (Action::Select, &keybinds.select),
        (Action::ToggleHelp, &keybinds.toggle_help),
        (Action::Move, &keybinds.move_torrent),
    ];

    for (action, keybind) in actions {
        if matches_keybind(&key_event, keybind) {
            return Ok(Some(action));
        }
    }
    Ok(None)
}

/// Handles the updates of [`App`].
#[tracing::instrument]
pub async fn update(app: &mut App<'_>, action: Action) -> Result<()> {
    let span = info_span!("update");
    let _enter = span.enter();
    event!(Level::INFO, "updating app with action: {:?}", action);
    match action {
        Action::Quit => app.quit(),
        Action::NextTab => app.next_tab(),
        Action::PrevTab => app.prev_tab(),
        Action::NextTorrent => app.next(),
        Action::PrevTorrent => app.previous(),
        Action::SwitchTab(x) => app.switch_tab(x as usize),
        Action::ToggleHelp => app.toggle_help(),
        Action::ToggleTorrent => app.toggle_torrents().await?,
        Action::ToggleAll => app.torrents.toggle_all().await?,
        Action::PauseAll => app.torrents.stop_all().await?,
        Action::StartAll => app.torrents.start_all().await?,
        Action::Move => app.prepare_move_action(),
        Action::Delete(x) => app.delete(x).await?,
        Action::Rename => unimplemented!(),
        Action::Select => app.select(),
        Action::Submit => app.move_torrent().await?,
        Action::Cancel => {
            app.input.clear();
            app.input_mode = false;
        }
    }
    Ok(())
}

/// Check if a KeyEvent matches a configured keybind string
fn matches_keybind(event: &KeyEvent, config_key: &Option<String>) -> bool {
    let Some(key_str) = config_key else {
        return false;
    };

    let (modifiers, key_code) = parse_keybind(key_str);
    let Some(key_code) = key_code else {
        return false;
    };

    event.code == key_code && event.modifiers == modifiers
}

fn parse_keybind(key_str: &str) -> (KeyModifiers, Option<KeyCode>) {
    let mut modifiers = KeyModifiers::NONE;
    let mut key_code = None;

    for part in key_str.split('+') {
        match part.trim().to_lowercase().as_str() {
            "ctrl" => modifiers.insert(KeyModifiers::CONTROL),
            "alt" => modifiers.insert(KeyModifiers::ALT),
            "shift" => modifiers.insert(KeyModifiers::SHIFT),
            key @ ("esc" | "enter" | "left" | "right" | "up" | "down" | "tab" | "backspace"
            | "delete" | "home" | "end" | "pageup" | "pagedown" | "null" | "insert") => {
                key_code = Some(match key {
                    "esc" => KeyCode::Esc,
                    "enter" => KeyCode::Enter,
                    "left" => KeyCode::Left,
                    "right" => KeyCode::Right,
                    "up" => KeyCode::Up,
                    "down" => KeyCode::Down,
                    "tab" => KeyCode::Tab,
                    "backspace" => KeyCode::Backspace,
                    "delete" => KeyCode::Delete,
                    "home" => KeyCode::Home,
                    "end" => KeyCode::End,
                    "pageup" => KeyCode::PageUp,
                    "pagedown" => KeyCode::PageDown,
                    "null" => KeyCode::Null,
                    "insert" => KeyCode::Insert,
                    _ => unreachable!(),
                });
            }
            f_key if f_key.starts_with('f') => {
                if let Ok(num) = f_key[1..].parse::<u8>() {
                    key_code = Some(KeyCode::F(num));
                }
            }
            single_char if single_char.len() == 1 => {
                if let Some(c) = single_char.chars().next() {
                    key_code = Some(KeyCode::Char(c));
                }
            }
            _ => return (modifiers, None),
        }
    }
    (modifiers, key_code)
}
