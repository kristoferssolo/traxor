use crate::app::{App, action::Action};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use thiserror::Error;
use tracing::{debug, info};

#[tracing::instrument(name = "Handling input", skip(app))]
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
///
/// # Errors
///
/// TODO: add error types
#[tracing::instrument(name = "Getting action", skip(app))]
pub async fn get_action(key_event: KeyEvent, app: &mut App<'_>) -> Result<Option<Action>> {
    if app.input_mode {
        return handle_input(key_event, app).await;
    }

    debug!("handling key event: {:?}", key_event);

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
///
/// # Errors
///
/// TODO: add error types
#[tracing::instrument(name = "Update", skip(app))]
pub async fn update(app: &mut App<'_>, action: Action) -> Result<()> {
    info!("updating app with action: {}", action);
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

/// Check if a [`KeyEvent`] matches a configured keybind string
fn matches_keybind(event: &KeyEvent, config_key: &str) -> bool {
    parse_keybind(config_key)
        .map(|parsed_ev| parsed_ev == *event)
        .unwrap_or(false)
}

#[derive(Debug, Error)]
pub enum ParseKeybingError {
    /// No “main” key was found (e.g. the user only wrote modifiers).
    #[error("no main key was found in input")]
    NoKeyCode,
    /// An unrecognized token was encountered.
    #[error("unrecognized token `{0}`")]
    UnknownPart(String),
}

fn parse_keybind(key_str: &str) -> Result<KeyEvent, ParseKeybingError> {
    let mut modifiers = KeyModifiers::NONE;
    let mut key_code = None;

    for raw in key_str.split('+') {
        let part = raw.trim();
        if part.is_empty() {
            if raw.contains(' ') {
                key_code = Some(KeyCode::Char(' '));
            }
            continue;
        }
        let low = part.to_lowercase();
        match low.as_str() {
            // modifiers
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            "alt" | "option" => modifiers |= KeyModifiers::ALT,

            // named keys
            "enter" => key_code = Some(KeyCode::Enter),
            "tab" => key_code = Some(KeyCode::Tab),
            "backspace" => key_code = Some(KeyCode::Backspace),
            "delete" => key_code = Some(KeyCode::Delete),
            "insert" => key_code = Some(KeyCode::Insert),
            "home" => key_code = Some(KeyCode::Home),
            "end" => key_code = Some(KeyCode::End),
            "pageup" | "page_up" => key_code = Some(KeyCode::PageUp),
            "pagedown" | "page_down" => key_code = Some(KeyCode::PageDown),
            "up" => key_code = Some(KeyCode::Up),
            "down" => key_code = Some(KeyCode::Down),
            "left" => key_code = Some(KeyCode::Left),
            "right" => key_code = Some(KeyCode::Right),
            "esc" | "escape" => key_code = Some(KeyCode::Esc),
            "space" => key_code = Some(KeyCode::Char(' ')),
            "null" => key_code = Some(KeyCode::Null),

            // symbol names
            "plus" => key_code = Some(KeyCode::Char('+')),
            "minus" => key_code = Some(KeyCode::Char('-')),
            "equals" | "equal" => key_code = Some(KeyCode::Char('=')),
            "comma" => key_code = Some(KeyCode::Char(',')),
            "dot" | "period" => key_code = Some(KeyCode::Char('.')),
            "semicolon" => key_code = Some(KeyCode::Char(';')),
            "slash" | "forward_slash" => key_code = Some(KeyCode::Char('/')),
            "backslash" => key_code = Some(KeyCode::Char('\\')),
            "tilde" => key_code = Some(KeyCode::Char('~')),
            "grave" | "backtick" => key_code = Some(KeyCode::Char('`')),
            "quote" => key_code = Some(KeyCode::Char('"')),
            "apostrophe" => key_code = Some(KeyCode::Char('\'')),

            // function keys F1...F<N>
            f if f.starts_with('f') && f.len() > 1 => {
                let num_str = &f[1..];
                match num_str.parse::<u8>() {
                    Ok(n) => key_code = Some(KeyCode::F(n)),
                    Err(_) => return Err(ParseKeybingError::UnknownPart(part.to_owned())),
                }
            }

            // single‐character fallback
            _ if part.len() == 1 => {
                if let Some(ch) = part.chars().next() {
                    key_code = Some(KeyCode::Char(ch));
                }
            }

            // unknown token
            other => return Err(ParseKeybingError::UnknownPart(other.to_owned())),
        }
    }

    key_code
        .map(|kc| KeyEvent::new(kc, modifiers))
        .ok_or(ParseKeybingError::NoKeyCode)
}
