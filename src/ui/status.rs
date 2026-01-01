use crate::app::{
    App, InputMode,
    utils::{filesize::FileSize, netspeed::NetSpeed},
};
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph},
};

#[allow(clippy::too_many_lines)]
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let torrents = &app.torrents.torrents;

    // Aggregate stats
    let total_down_speed = torrents.iter().filter_map(|t| t.rate_download).sum::<i64>();
    let total_up_speed = torrents.iter().filter_map(|t| t.rate_upload).sum::<i64>();
    let total_downloaded = torrents
        .iter()
        .filter_map(|t| t.downloaded_ever)
        .sum::<u64>();
    let total_uploaded = torrents.iter().filter_map(|t| t.uploaded_ever).sum::<i64>();

    let down_speed = NetSpeed::new(total_down_speed.unsigned_abs());
    let up_speed = NetSpeed::new(total_up_speed.unsigned_abs());
    let downloaded = FileSize::new(total_downloaded);
    let uploaded = FileSize::new(total_uploaded.unsigned_abs());

    let total = app.torrents.len();
    let filtered = app.filtered_torrents().len();
    let selected_count = app.torrents.selected.len();

    let active_filter = app.active_filter();

    let mode_text = match app.input_mode {
        InputMode::Move => Some("MOVE".to_string()),
        InputMode::Rename => Some("RENAME".to_string()),
        InputMode::Filter => Some(format!("Filter: {active_filter}")),
        InputMode::ConfirmDelete(_) => Some("DELETE".to_string()),
        InputMode::None if !active_filter.is_empty() => Some(format!("Filter: {active_filter}")),
        InputMode::None => None,
    };

    let keybinds = match app.input_mode {
        InputMode::None if !app.filter_text.is_empty() => vec![
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" Clear │ "),
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw(" Help"),
        ],
        InputMode::None => vec![
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw(" Help │ "),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw(" Search"),
        ],
        InputMode::Move | InputMode::Rename => vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Submit │ "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ],
        InputMode::Filter => vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Confirm │ "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ],
        InputMode::ConfirmDelete(_) => vec![
            Span::styled("y", Style::default().fg(Color::Green)),
            Span::raw(" Confirm │ "),
            Span::styled("n", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ],
    };

    // Build right side with colored spans
    let count_style = if selected_count > 0 {
        Style::default().fg(Color::Magenta).bold()
    } else if !active_filter.is_empty() {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };

    let count_text = if selected_count > 0 {
        format!("{selected_count}/{total}")
    } else if !active_filter.is_empty() {
        format!("{filtered}/{total}")
    } else {
        format!("{total}")
    };

    let down_style = if total_down_speed > 0 {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let up_style = if total_up_speed > 0 {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let right_spans = vec![
        Span::styled(count_text, count_style),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("↓{down_speed}"), down_style),
        Span::raw(" "),
        Span::styled(format!("↑{up_speed}"), up_style),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("D:", Style::default().fg(Color::DarkGray)),
        Span::raw(format!("{downloaded} ")),
        Span::styled("U:", Style::default().fg(Color::DarkGray)),
        Span::raw(format!("{uploaded} ")),
    ];

    // Calculate widths
    let available_width = area.width.saturating_sub(2) as usize;
    let left_len = keybinds
        .iter()
        .map(|s| s.content.chars().count())
        .sum::<usize>()
        + 1;
    let right_len = right_spans
        .iter()
        .map(|s| s.content.chars().count())
        .sum::<usize>();

    let mut spans = vec![Span::raw(" ")];
    spans.extend(keybinds);

    if left_len + right_len < available_width {
        let padding = available_width.saturating_sub(left_len + right_len);
        spans.push(Span::raw(" ".repeat(padding)));
        spans.extend(right_spans);
    }

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    if let Some(ref mode) = mode_text {
        block = block
            .title(format!(" {mode} "))
            .title_style(Style::default().fg(Color::Yellow).bold());
    }

    let paragraph = Paragraph::new(Line::from(spans)).block(block);

    frame.render_widget(paragraph, area);
}
