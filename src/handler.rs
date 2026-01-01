use crate::app::{App, InputMode, action::Action};
use crate::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use thiserror::Error;
use tracing::{debug, info};

#[tracing::instrument(name = "Handling input", skip(app))]
async fn handle_input(key_event: KeyEvent, app: &mut App) -> Result<Option<Action>> {
    // Handle confirmation dialogs separately
    if matches!(app.input_mode, InputMode::ConfirmDelete(_)) {
        return match key_event.code {
            KeyCode::Char('y' | 'Y') => Ok(Some(Action::ConfirmYes)),
            KeyCode::Char('n' | 'N') | KeyCode::Esc => Ok(Some(Action::Cancel)),
            _ => Ok(None),
        };
    }

    match key_event.code {
        KeyCode::Enter => Ok(Some(Action::Submit)),
        KeyCode::Tab => {
            app.complete_input().await?;
            Ok(None)
        }
        KeyCode::Char(ch) => {
            app.input_handler.insert_char(ch);
            Ok(None)
        }
        KeyCode::Backspace => {
            app.input_handler.delete_char();
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
/// Returns an error if input handling fails.
#[tracing::instrument(name = "Getting action", skip(app))]
pub async fn get_action(key_event: KeyEvent, app: &mut App) -> Result<Option<Action>> {
    if app.input_mode != InputMode::None {
        return handle_input(key_event, app).await;
    }

    debug!("handling key event: {:?}", key_event);

    let keybinds = &app.config.keybinds;

    Ok([
        (Action::Quit, &keybinds.quit),
        (Action::NextTab, &keybinds.next_tab),
        (Action::PrevTab, &keybinds.prev_tab),
        (Action::NextTorrent, &keybinds.next_torrent),
        (Action::PrevTorrent, &keybinds.prev_torrent),
        (Action::SwitchTab(0), &keybinds.switch_tab_1),
        (Action::SwitchTab(1), &keybinds.switch_tab_2),
        (Action::SwitchTab(2), &keybinds.switch_tab_3),
        (Action::SwitchTab(3), &keybinds.switch_tab_4),
        (Action::SwitchTab(4), &keybinds.switch_tab_5),
        (Action::SwitchTab(5), &keybinds.switch_tab_6),
        (Action::SwitchTab(6), &keybinds.switch_tab_7),
        (Action::SwitchTab(7), &keybinds.switch_tab_8),
        (Action::SwitchTab(8), &keybinds.switch_tab_9),
        (Action::SwitchTab(9), &keybinds.switch_tab_10),
        (Action::ToggleTorrent, &keybinds.toggle_torrent),
        (Action::ToggleAll, &keybinds.toggle_all),
        (Action::Delete(false), &keybinds.delete),
        (Action::Delete(true), &keybinds.delete_force),
        (Action::Select, &keybinds.select),
        (Action::ToggleHelp, &keybinds.toggle_help),
        (Action::Move, &keybinds.move_torrent),
        (Action::Rename, &keybinds.rename_torrent),
    ]
    .into_iter()
    .find_map(|(action, keybind)| matches_keybind(&key_event, keybind).then_some(action)))
}

/// Handles the updates of [`App`].
///
/// # Errors
///
/// Returns an error if the action fails.
#[tracing::instrument(name = "Update", skip(app))]
pub async fn update(app: &mut App, action: Action) -> Result<()> {
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
        Action::Rename => app.prepare_rename_action(),
        Action::Delete(delete_local_data) => app.prepare_delete(delete_local_data),
        Action::Select => app.select(),
        Action::Submit => match app.input_mode {
            InputMode::Move => app.move_torrent().await?,
            InputMode::Rename => app.rename_torrent().await?,
            InputMode::None | InputMode::ConfirmDelete(_) => {}
        },
        Action::ConfirmYes => app.confirm_delete().await?,
        Action::Cancel => {
            app.input_handler.clear();
            app.input_mode = InputMode::None;
        }
    }
    Ok(())
}

/// Check if a [`KeyEvent`] matches a configured keybind string
fn matches_keybind(event: &KeyEvent, config_key: &str) -> bool {
    parse_keybind(config_key).is_ok_and(|parsed| parsed == *event)
}

#[derive(Debug, Error)]
pub enum ParseKeybindError {
    /// No "main" key was found (e.g. the user only wrote modifiers).
    #[error("no main key was found in input")]
    NoKeyCode,
    /// An unrecognized token was encountered.
    #[error("unrecognized token `{0}`")]
    UnknownPart(String),
}

fn parse_keybind(key_str: &str) -> std::result::Result<KeyEvent, ParseKeybindError> {
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

        match part.to_ascii_lowercase().as_str() {
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
            f if f.starts_with('f') => {
                key_code =
                    Some(KeyCode::F(f[1..].parse().map_err(|_| {
                        ParseKeybindError::UnknownPart(part.to_owned())
                    })?));
            }

            // single-character fallback
            _ if part.len() == 1 => {
                key_code = part.chars().next().map(KeyCode::Char);
            }

            // unknown token
            other => return Err(ParseKeybindError::UnknownPart(other.to_owned())),
        }
    }

    key_code
        .map(|kc| KeyEvent::new(kc, modifiers))
        .ok_or(ParseKeybindError::NoKeyCode)
}
