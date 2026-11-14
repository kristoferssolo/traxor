use crate::app::action::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeybindsConfig {
    pub quit: String,
    pub next_tab: String,
    pub prev_tab: String,
    pub next_torrent: String,
    pub prev_torrent: String,
    pub switch_tab_1: String,
    pub switch_tab_2: String,
    pub switch_tab_3: String,
    pub switch_tab_4: String,
    pub switch_tab_5: String,
    pub switch_tab_6: String,
    pub switch_tab_7: String,
    pub switch_tab_8: String,
    pub switch_tab_9: String,
    pub switch_tab_10: String,
    pub toggle_torrent: String,
    pub toggle_all: String,
    pub delete: String,
    pub delete_force: String,
    pub select: String,
    pub toggle_help: String,
    pub move_torrent: String,
    pub rename_torrent: String,
    pub filter: String,
    pub clear_filter: String,
}

impl KeybindsConfig {
    /// Parse every configured keybind into a [`Keymap`].
    ///
    /// # Errors
    ///
    /// Returns an error when a keybind string cannot be parsed or when two
    /// keybinds resolve to the same key.
    pub fn keymap(&self) -> Result<Keymap, KeybindError> {
        // Config key, bound action, and configured key string.
        let entries = [
            ("quit", Action::Quit, &self.quit),
            ("next_tab", Action::NextTab, &self.next_tab),
            ("prev_tab", Action::PrevTab, &self.prev_tab),
            ("next_torrent", Action::NextTorrent, &self.next_torrent),
            ("prev_torrent", Action::PrevTorrent, &self.prev_torrent),
            ("switch_tab_1", Action::SwitchTab(0), &self.switch_tab_1),
            ("switch_tab_2", Action::SwitchTab(1), &self.switch_tab_2),
            ("switch_tab_3", Action::SwitchTab(2), &self.switch_tab_3),
            ("switch_tab_4", Action::SwitchTab(3), &self.switch_tab_4),
            ("switch_tab_5", Action::SwitchTab(4), &self.switch_tab_5),
            ("switch_tab_6", Action::SwitchTab(5), &self.switch_tab_6),
            ("switch_tab_7", Action::SwitchTab(6), &self.switch_tab_7),
            ("switch_tab_8", Action::SwitchTab(7), &self.switch_tab_8),
            ("switch_tab_9", Action::SwitchTab(8), &self.switch_tab_9),
            ("switch_tab_10", Action::SwitchTab(9), &self.switch_tab_10),
            (
                "toggle_torrent",
                Action::ToggleTorrent,
                &self.toggle_torrent,
            ),
            ("toggle_all", Action::ToggleAll, &self.toggle_all),
            ("delete", Action::Delete(false), &self.delete),
            ("delete_force", Action::Delete(true), &self.delete_force),
            ("select", Action::Select, &self.select),
            ("toggle_help", Action::ToggleHelp, &self.toggle_help),
            ("move_torrent", Action::Move, &self.move_torrent),
            ("rename_torrent", Action::Rename, &self.rename_torrent),
            ("filter", Action::Filter, &self.filter),
            ("clear_filter", Action::ClearFilter, &self.clear_filter),
        ];

        let mut bindings: Vec<(&'static str, KeyEvent, Action)> = Vec::with_capacity(entries.len());
        for (name, action, value) in entries {
            let event = parse_keybind(value).map_err(|source| KeybindError::Invalid {
                name,
                value: value.to_owned(),
                source,
            })?;

            if let Some((existing, ..)) = bindings.iter().find(|(_, bound, _)| *bound == event) {
                return Err(KeybindError::Duplicate {
                    first: existing,
                    second: name,
                    value: value.to_owned(),
                });
            }

            bindings.push((name, event, action));
        }

        Ok(Keymap(
            bindings
                .into_iter()
                .map(|(_, event, action)| (event, action))
                .collect(),
        ))
    }
}

/// Parsed keybinds, built once at config load.
#[derive(Debug, Clone, Default)]
pub struct Keymap(Vec<(KeyEvent, Action)>);

impl Keymap {
    /// Return the action bound to `event`, if any.
    #[must_use]
    pub fn action(&self, event: &KeyEvent) -> Option<Action> {
        self.0
            .iter()
            .find(|(bound, _)| bound == event)
            .map(|(_, action)| action.clone())
    }
}

#[derive(Debug, Error)]
pub enum KeybindError {
    #[error("invalid keybinds.{name} {value:?}: {source}")]
    Invalid {
        name: &'static str,
        value: String,
        source: ParseKeybindError,
    },
    #[error("duplicate keybinds.{second} {value:?}: already bound by keybinds.{first}")]
    Duplicate {
        first: &'static str,
        second: &'static str,
        value: String,
    },
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

fn parse_keybind(key_str: &str) -> Result<KeyEvent, ParseKeybindError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok_eq};

    #[test]
    fn parses_keybind_syntax() {
        for (input, code, modifiers) in [
            ("D", KeyCode::Char('D'), KeyModifiers::NONE),
            (" ", KeyCode::Char(' '), KeyModifiers::NONE),
            ("ctrl+ ", KeyCode::Char(' '), KeyModifiers::CONTROL),
            (
                "Ctrl+Alt+x",
                KeyCode::Char('x'),
                KeyModifiers::CONTROL | KeyModifiers::ALT,
            ),
            ("escape", KeyCode::Esc, KeyModifiers::NONE),
            ("page_down", KeyCode::PageDown, KeyModifiers::NONE),
            ("slash", KeyCode::Char('/'), KeyModifiers::NONE),
            ("F12", KeyCode::F(12), KeyModifiers::NONE),
        ] {
            assert_ok_eq!(parse_keybind(input), KeyEvent::new(code, modifiers));
        }
    }

    #[test]
    fn rejects_invalid_keybind_syntax() {
        for input in ["ctrl+", "ctrl+bogus", "fx"] {
            assert_err!(parse_keybind(input));
        }
    }
}
