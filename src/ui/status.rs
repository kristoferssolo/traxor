use crate::app::{
    App, InputMode,
    utils::{filesize::FileSize, netspeed::NetSpeed},
};
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let torrents = &app.torrents.torrents;

    // Aggregate stats
    let total_down_speed: i64 = torrents.iter().filter_map(|t| t.rate_download).sum();
    let total_up_speed: i64 = torrents.iter().filter_map(|t| t.rate_upload).sum();
    let total_downloaded: u64 = torrents.iter().filter_map(|t| t.downloaded_ever).sum();
    let total_uploaded: i64 = torrents.iter().filter_map(|t| t.uploaded_ever).sum();

    let down_speed = NetSpeed::new(total_down_speed.unsigned_abs());
    let up_speed = NetSpeed::new(total_up_speed.unsigned_abs());
    let downloaded = FileSize::new(total_downloaded);
    let uploaded = FileSize::new(total_uploaded.unsigned_abs());

    let total = app.torrents.len();
    let selected_count = app.torrents.selected.len();

    let count_info = if selected_count > 0 {
        format!("{selected_count}/{total}")
    } else {
        format!("{total}")
    };

    let mode_text = match app.input_mode {
        InputMode::Move => Some("MOVE"),
        InputMode::Rename => Some("RENAME"),
        InputMode::ConfirmDelete(_) => Some("DELETE"),
        InputMode::None => None,
    };

    let keybinds = match app.input_mode {
        InputMode::None => "? Help",
        InputMode::Move | InputMode::Rename => "Enter Submit │ Esc Cancel",
        InputMode::ConfirmDelete(_) => "y Confirm │ n Cancel",
    };

    let left = format!(" {keybinds}");
    let right =
        format!("{count_info} │ ↓ {down_speed} ↑ {up_speed} │ D: {downloaded} U: {uploaded} ");

    let available_width = area.width.saturating_sub(2) as usize;
    let left_len = left.chars().count();
    let right_len = right.chars().count();

    let content = if left_len + right_len < available_width {
        let padding = available_width.saturating_sub(left_len + right_len);
        format!("{left}{}{right}", " ".repeat(padding))
    } else if left_len < available_width {
        left
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
