use crate::{
    app::{
        App, InputMode,
        utils::{filesize::FileSize, netspeed::NetSpeed},
    },
    ui::help::display_key,
};
use ratatui::{
    buffer::CellWidth,
    prelude::*,
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph},
};

/// Blank column before the left-side text.
const LEADING_SPACE: usize = 1;
/// Minimum blank columns between the left-side text and the stats.
const MIN_GAP: usize = 1;
const ELLIPSIS: &str = "…";

#[allow(clippy::too_many_lines)]
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let torrents = &app.torrents.torrents;

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
        InputMode::None if app.show_help => Some("HELP".to_string()),
        InputMode::Move => Some("MOVE".to_string()),
        InputMode::Rename => Some("RENAME".to_string()),
        InputMode::Filter => Some(format!("Filter: {active_filter}")),
        InputMode::ConfirmDelete(_) => Some("DELETE".to_string()),
        InputMode::None if !active_filter.is_empty() => Some(format!("Filter: {active_filter}")),
        InputMode::None => None,
    };

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

    // Stats keep their width; the hints or error get the rest of the line.
    let available_width = usize::from(area.width.saturating_sub(2));
    let right_width = spans_width(&right_spans);
    let stats_fit = LEADING_SPACE + MIN_GAP + right_width <= available_width;
    let left_max_width = if stats_fit {
        available_width - LEADING_SPACE - MIN_GAP - right_width
    } else {
        available_width.saturating_sub(LEADING_SPACE)
    };
    // An error notice takes the place of the keybind hints until the next key press.
    let left_spans = app.status_message.as_ref().map_or_else(
        || keybind_hints(app, if stats_fit { left_max_width } else { 0 }),
        |message| {
            vec![Span::styled(
                message.text.as_str(),
                Style::default().fg(Color::Red),
            )]
        },
    );
    let left_spans = truncate_spans(left_spans, left_max_width);
    let left_width = spans_width(&left_spans);

    let mut spans = vec![Span::raw(" ")];
    spans.extend(left_spans);

    if stats_fit {
        let padding = available_width - LEADING_SPACE - left_width - right_width;
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

/// Total display width of `spans` in terminal columns.
fn spans_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(rendered_width).sum()
}

/// Cells `span` occupies once rendered: the sum of its graphemes' widths, as ratatui draws
/// them. Measuring the whole string can differ: `"لا"` measures one cell as a string but its
/// two graphemes fill two cells.
fn rendered_width(span: &Span<'_>) -> usize {
    span.styled_graphemes(Style::default())
        .map(|grapheme| grapheme_width(grapheme.symbol))
        .sum()
}

/// Cells ratatui's buffer gives `symbol`, which adjusts some graphemes (e.g. halfwidth
/// katakana sound marks) that plain Unicode width measures differently.
fn grapheme_width(symbol: &str) -> usize {
    usize::from(symbol.cell_width())
}

/// Shortens `spans` to at most `max_width` terminal columns.
///
/// Cuts at grapheme boundaries and ends with `…` in the style of the cut span
/// when anything was removed.
fn truncate_spans(spans: Vec<Span<'_>>, max_width: usize) -> Vec<Span<'_>> {
    if spans_width(&spans) <= max_width {
        return spans;
    }
    let Some(mut remaining) = max_width.checked_sub(grapheme_width(ELLIPSIS)) else {
        return Vec::new();
    };

    let mut truncated = Vec::new();
    for span in spans {
        let span_width = rendered_width(&span);
        if span_width <= remaining {
            remaining -= span_width;
            truncated.push(span);
            continue;
        }

        let mut content = String::new();
        for grapheme in span.styled_graphemes(Style::default()) {
            let symbol_width = grapheme_width(grapheme.symbol);
            if symbol_width > remaining {
                break;
            }
            remaining -= symbol_width;
            content.push_str(grapheme.symbol);
        }
        truncated.push(Span::styled(content, span.style));
        truncated.push(Span::styled(ELLIPSIS, span.style));
        break;
    }
    truncated
}

/// Keybind hints for the left side of the status bar.
///
/// `room` is the width left beside the stats, or zero when the stats do not fit. In normal
/// mode the sort hints replace the search hint when they fit.
fn keybind_hints(app: &App, room: usize) -> Vec<Span<'_>> {
    match app.input_mode {
        InputMode::None if app.show_help => vec![
            Span::styled(
                display_key(&app.config.keybinds().toggle_help),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" Close help"),
        ],
        InputMode::None if !app.filter_text.is_empty() => vec![
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" Clear │ "),
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw(" Help"),
        ],
        InputMode::None => {
            let kb = app.config.keybinds();
            let sort_hints = vec![
                Span::styled(kb.sort_column.as_str(), Style::default().fg(Color::Yellow)),
                Span::raw(" Sort │ "),
                Span::styled(kb.sort_reverse.as_str(), Style::default().fg(Color::Yellow)),
                Span::raw(" Reverse │ "),
                Span::styled("?", Style::default().fg(Color::Yellow)),
                Span::raw(" Help"),
            ];
            if spans_width(&sort_hints) <= room {
                return sort_hints;
            }
            vec![
                Span::styled("?", Style::default().fg(Color::Yellow)),
                Span::raw(" Help │ "),
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::raw(" Search"),
            ]
        }
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_some_eq;

    fn text(spans: &[Span<'_>]) -> String {
        spans.iter().map(|span| span.content.as_ref()).collect()
    }

    #[test]
    fn truncate_spans_keeps_text_that_fits() {
        let spans = vec![Span::raw("ab"), Span::raw("cd")];

        let truncated = truncate_spans(spans.clone(), 4);

        assert_eq!(truncated, spans);
    }

    #[test]
    fn truncate_spans_cuts_across_spans_with_ellipsis() {
        let red = Style::default().fg(Color::Red);
        let spans = vec![Span::raw("ab"), Span::styled("cdef", red)];

        let truncated = truncate_spans(spans, 5);

        assert_eq!(text(&truncated), "abcd…");
        assert_eq!(spans_width(&truncated), 5);
        assert_some_eq!(truncated.last().map(|span| span.style), red);
    }

    #[test]
    fn truncate_spans_never_splits_wide_characters() {
        let spans = vec![Span::raw("错误信息")];

        let truncated = truncate_spans(spans, 6);

        assert_eq!(text(&truncated), "错误…");
        assert_eq!(spans_width(&truncated), 5);
    }

    #[test]
    fn widths_follow_rendered_graphemes_for_ligatures() {
        let spans = vec![Span::raw("لا".repeat(3))];

        assert_eq!(spans_width(&spans), 6);
        assert_eq!(text(&truncate_spans(spans, 5)), "لالا…");
    }

    #[test]
    fn truncate_spans_returns_nothing_without_room_for_ellipsis() {
        let truncated = truncate_spans(vec![Span::raw("ab")], 0);

        assert_eq!(truncated, Vec::<Span<'_>>::new());
    }
}
