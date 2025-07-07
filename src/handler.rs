use crate::app::{action::Action, App};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events of [`App`].
pub fn get_action(key_event: KeyEvent) -> Option<Action> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => Some(Action::Quit),

        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => match key_event.modifiers {
            KeyModifiers::CONTROL => Some(Action::Quit),
            _ => None,
        },
        KeyCode::Char('l') | KeyCode::Right => Some(Action::NextTab),
        KeyCode::Char('h') | KeyCode::Left => Some(Action::PrevTab),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::NextTorrent),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::PrevTorrent),
        KeyCode::Char('1') => Some(Action::SwitchTab(0)),
        KeyCode::Char('2') => Some(Action::SwitchTab(1)),
        KeyCode::Char('3') => Some(Action::SwitchTab(2)),
        KeyCode::Char('t') | KeyCode::Enter | KeyCode::Menu => Some(Action::ToggleTorrent),
        KeyCode::Char('a') => Some(Action::ToggleAll),
        KeyCode::Char('d') => Some(Action::Delete(false)),
        KeyCode::Char('D') => Some(Action::Delete(true)),
        KeyCode::Char(' ') => Some(Action::Select),
        KeyCode::Char('?') => Some(Action::ToggleHelp),
        _ => None,
    }
}

/// Handles the updates of [`App`].
pub async fn update(app: &mut App<'_>, action: Action) -> anyhow::Result<()> {
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
