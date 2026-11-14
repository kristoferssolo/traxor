use crate::{
    app::{App, InputMode, action::Action},
    error::Result,
    ui::hit_test::{tab_at, table_row_at},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{layout::Position, text::Line};
use tracing::{debug, error, info};

#[tracing::instrument(name = "Handling input", skip(app))]
async fn handle_input(key_event: KeyEvent, app: &mut App) -> Result<Option<Action>> {
    // Handle confirmation dialogs separately
    if matches!(app.input_mode, InputMode::ConfirmDelete(_)) {
        return match key_event.code {
            KeyCode::Char('y' | 'Y') | KeyCode::Enter => Ok(Some(Action::ConfirmYes)),
            KeyCode::Char('n' | 'N') | KeyCode::Esc => Ok(Some(Action::Cancel)),
            _ => Ok(None),
        };
    }

    let ctrl = key_event.modifiers.contains(KeyModifiers::CONTROL);
    let input = &mut app.input_handler;
    match key_event.code {
        KeyCode::Enter => return Ok(Some(Action::Submit)),
        KeyCode::Esc => return Ok(Some(Action::Cancel)),
        KeyCode::Tab if app.input_mode == InputMode::Move => app.complete_input().await?,
        KeyCode::Char('a') if ctrl => input.move_cursor_to_start(),
        KeyCode::Char('e') if ctrl => input.move_cursor_to_end(),
        KeyCode::Char('w') if ctrl => input.delete_word_before_cursor(),
        KeyCode::Char('u') if ctrl => input.delete_to_start(),
        KeyCode::Char(ch)
            if !key_event
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            input.insert_char(ch);
        }
        KeyCode::Backspace => input.delete_char(),
        KeyCode::Delete => input.delete_char_forward(),
        KeyCode::Left => input.move_cursor_left(),
        KeyCode::Right => input.move_cursor_right(),
        KeyCode::Home => input.move_cursor_to_start(),
        KeyCode::End => input.move_cursor_to_end(),
        _ => {}
    }
    Ok(None)
}

/// Handles a key press end to end: maps it to an [`Action`] and applies it.
///
/// Clears the previous status message first. Errors are logged and shown via
/// [`App::show_error`] instead of being returned, so a failed action or input
/// never ends the app.
#[tracing::instrument(name = "Handling key event", skip(app))]
pub async fn handle_key_event(key_event: KeyEvent, app: &mut App) {
    app.clear_status_message();
    let result = match get_action(key_event, app).await {
        Ok(Some(action)) => update(app, action).await,
        Ok(None) => Ok(()),
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        error!("Failed to handle key event: {error}");
        app.show_error(error.to_string());
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

    // Close help popup with Esc
    if app.show_help && key_event.code == KeyCode::Esc {
        return Ok(Some(Action::ToggleHelp));
    }

    debug!("handling key event: {:?}", key_event);

    Ok(app.config.keymap().action(&key_event))
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
        Action::Move => app.prepare_move_action(),
        Action::Rename => app.prepare_rename_action(),
        Action::Filter => app.start_filter(),
        Action::ClearFilter => app.clear_filter(),
        Action::Delete(delete_local_data) => app.prepare_delete(delete_local_data),
        Action::Select => app.select(),
        Action::Submit => match app.input_mode {
            InputMode::Move => app.move_torrent().await?,
            InputMode::Rename => app.rename_torrent().await?,
            InputMode::Filter => app.apply_filter(),
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

/// Handles mouse events of [`App`].
///
/// Scrolling over the table moves the highlight, and a left click highlights a torrent row or
/// switches to a tab. Ignored while a popup or input dialog is open.
pub fn handle_mouse(mouse_event: MouseEvent, app: &mut App) {
    if app.show_help || app.input_mode != InputMode::None {
        return;
    }

    let position = Position::new(mouse_event.column, mouse_event.row);
    match mouse_event.kind {
        MouseEventKind::ScrollDown if app.areas.table.contains(position) => app.next(),
        MouseEventKind::ScrollUp if app.areas.table.contains(position) => app.previous(),
        MouseEventKind::Down(MouseButton::Left) => click(app, position),
        _ => {}
    }
}

fn click(app: &mut App, position: Position) {
    let areas = &app.areas;
    if let Some(row) = table_row_at(
        areas.table,
        areas.table_offset,
        areas.table_rows.len(),
        position,
    ) {
        // Highlight the torrent that was displayed in that row, wherever it is now. Ignore the
        // click if a refresh since the last render removed it.
        let index = areas.table_rows[row].and_then(|id| {
            app.filtered_torrents()
                .iter()
                .position(|torrent| torrent.id == Some(id))
        });
        if index.is_some() {
            app.state.select(index);
        }
        return;
    }

    let title_widths = app
        .tabs()
        .iter()
        .map(|tab| Line::from(tab.to_string()).width());
    if let Some(tab) = tab_at(app.areas.tabs, title_widths, position) {
        app.switch_tab(tab);
    }
}
