use crate::app::{App, InputMode};
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let total = app.torrents.len();
    let selected_count = app.torrents.selected.len();

    let left = if selected_count > 0 {
        format!(" {selected_count}/{total} selected")
    } else {
        format!(" {total} torrents")
    };

    let mode_text = match app.input_mode {
        InputMode::Move => Some("MOVE"),
        InputMode::Rename => Some("RENAME"),
        InputMode::ConfirmDelete(_) => Some("DELETE"),
        InputMode::None => None,
    };

    let keybinds = match app.input_mode {
        InputMode::None => "q Quit │ ? Help │ ↑↓ Navigate │ Space Select │ Enter Toggle",
        InputMode::Move | InputMode::Rename => "Enter Submit │ Esc Cancel │ Tab Complete",
        InputMode::ConfirmDelete(_) => "y Confirm │ n Cancel",
    };

    let right = format!("{keybinds} ");

    let available_width = area.width.saturating_sub(2) as usize;
    let left_len = left.chars().count();
    let right_len = right.chars().count();

    let content = if left_len + right_len < available_width {
        let padding = available_width.saturating_sub(left_len + right_len);
        format!("{left}{}{right}", " ".repeat(padding))
    } else {
        right
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    if let Some(mode) = mode_text {
        block = block
            .title(format!(" {mode} "))
            .title_style(Style::default().fg(Color::Yellow).bold());
    }

    let paragraph = Paragraph::new(Span::raw(content)).block(block);

    frame.render_widget(paragraph, area);
}
