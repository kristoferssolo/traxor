use crate::app::{action::Action, App};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tracing::{event, info_span, Level};

/// Handles the key events of [`App`].
#[tracing::instrument]
pub fn get_action(key_event: KeyEvent, app: &App) -> Option<Action> {
    let span = info_span!("get_action");
    let _enter = span.enter();
    event!(Level::INFO, "handling key event: {:?}", key_event);

    let config_keybinds = &app.config.keybinds;

    // Helper to check if a KeyEvent matches a configured keybind string
    let matches_keybind = |event: &KeyEvent, config_key: &Option<String>| {
        if let Some(key_str) = config_key {
            let parts: Vec<&str> = key_str.split('+').collect();
            let mut parsed_modifiers = KeyModifiers::NONE;
            let mut parsed_key_code = None;

            for part in &parts {
                match part.to_lowercase().as_str() {
                    "ctrl" => parsed_modifiers.insert(KeyModifiers::CONTROL),
                    "alt" => parsed_modifiers.insert(KeyModifiers::ALT),
                    "shift" => parsed_modifiers.insert(KeyModifiers::SHIFT),
                    "esc" => parsed_key_code = Some(KeyCode::Esc),
                    "enter" => parsed_key_code = Some(KeyCode::Enter),
                    "left" => parsed_key_code = Some(KeyCode::Left),
                    "right" => parsed_key_code = Some(KeyCode::Right),
                    "up" => parsed_key_code = Some(KeyCode::Up),
                    "down" => parsed_key_code = Some(KeyCode::Down),
                    "tab" => parsed_key_code = Some(KeyCode::Tab),
                    "backspace" => parsed_key_code = Some(KeyCode::Backspace),
                    "delete" => parsed_key_code = Some(KeyCode::Delete),
                    "home" => parsed_key_code = Some(KeyCode::Home),
                    "end" => parsed_key_code = Some(KeyCode::End),
                    "pageup" => parsed_key_code = Some(KeyCode::PageUp),
                    "pagedown" => parsed_key_code = Some(KeyCode::PageDown),
                    "null" => parsed_key_code = Some(KeyCode::Null),
                    "insert" => parsed_key_code = Some(KeyCode::Insert),
                    _ => {
                        if part.len() == 1 {
                            if let Some(c) = part.chars().next() {
                                parsed_key_code = Some(KeyCode::Char(c));
                            } else {
                                return false;
                            }
                        } else if part.starts_with("f") && part.len() > 1 {
                            if let Ok(f_num) = part[1..].parse::<u8>() {
                                parsed_key_code = Some(KeyCode::F(f_num));
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
            }

            if parsed_key_code.is_none() {
                return false;
            }

            event.code == parsed_key_code.unwrap() && event.modifiers == parsed_modifiers
        } else {
            false
        }
    };

    match key_event.code {
        _ if matches_keybind(&key_event, &config_keybinds.quit) => Some(Action::Quit),
        _ if matches_keybind(&key_event, &config_keybinds.next_tab) => Some(Action::NextTab),
        _ if matches_keybind(&key_event, &config_keybinds.prev_tab) => Some(Action::PrevTab),
        _ if matches_keybind(&key_event, &config_keybinds.next_torrent) => Some(Action::NextTorrent),
        _ if matches_keybind(&key_event, &config_keybinds.prev_torrent) => Some(Action::PrevTorrent),
        _ if matches_keybind(&key_event, &config_keybinds.switch_tab_1) => {
            Some(Action::SwitchTab(0))
        }
        _ if matches_keybind(&key_event, &config_keybinds.switch_tab_2) => {
            Some(Action::SwitchTab(1))
        }
        _ if matches_keybind(&key_event, &config_keybinds.switch_tab_3) => {
            Some(Action::SwitchTab(2))
        }
        _ if matches_keybind(&key_event, &config_keybinds.toggle_torrent) => {
            Some(Action::ToggleTorrent)
        }
        _ if matches_keybind(&key_event, &config_keybinds.toggle_all) => Some(Action::ToggleAll),
        _ if matches_keybind(&key_event, &config_keybinds.delete) => Some(Action::Delete(false)),
        _ if matches_keybind(&key_event, &config_keybinds.delete_force) => {
            Some(Action::Delete(true))
        }
        _ if matches_keybind(&key_event, &config_keybinds.select) => Some(Action::Select),
        _ if matches_keybind(&key_event, &config_keybinds.toggle_help) => Some(Action::ToggleHelp),
        _ => None,
    }
}

/// Handles the updates of [`App`].
#[tracing::instrument]
pub async fn update(app: &mut App<'_>, action: Action) -> color_eyre::eyre::Result<()> {
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
        Action::Move => unimplemented!(),
        Action::Delete(x) => app.delete(x).await?,
        Action::Rename => unimplemented!(),
        Action::Select => app.select(),
    }
    Ok(())
}
