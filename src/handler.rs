use crate::app::App;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> Result<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.next_tab();
        }
        KeyCode::Char('h') | KeyCode::Left => {
            app.prev_tab();
        }

        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        KeyCode::Char('1') => app.switch_tab(0),
        KeyCode::Char('2') => app.switch_tab(1),
        KeyCode::Char('3') => app.switch_tab(2),
        KeyCode::Char('4') => app.switch_tab(3),
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
