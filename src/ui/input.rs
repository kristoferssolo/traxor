use crate::app::{App, InputMode, types::DeleteTarget};
use ratatui::{
    buffer::CellWidth,
    prelude::*,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
};
use std::iter;
use unicode_segmentation::UnicodeSegmentation;

pub fn render(f: &mut Frame, app: &App) {
    match &app.input_mode {
        InputMode::Move | InputMode::Rename => render_text_input(f, app),
        InputMode::Filter => render_filter_input(f, app),
        InputMode::ConfirmDelete(target) => render_confirm_delete(f, target),
        InputMode::None => {}
    }
}

fn render_text_input(f: &mut Frame, app: &App) {
    let size = f.area();
    let input_area = Rect::new(
        size.width / 4,
        (size.height / 2).saturating_sub(1),
        size.width / 2,
        3,
    )
    .intersection(size);

    let title = match app.input_mode {
        InputMode::Move => "Move to",
        InputMode::Rename => "Rename",
        _ => return,
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(Clear, input_area);
    f.render_widget(block, input_area);

    let text_area = input_area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    let view = input_view(
        &app.input_handler.text,
        app.input_handler.cursor_position,
        text_area.width,
    );
    f.render_widget(Paragraph::new(view.visible), text_area);

    if app.input_mode == InputMode::Move {
        render_completion_popup(f, app, input_area);
    }

    if !text_area.is_empty() {
        f.set_cursor_position(Position::new(text_area.x + view.cursor_column, text_area.y));
    }
}

fn render_completion_popup(f: &mut Frame, app: &App, input_area: Rect) {
    let completions = app.input_handler.completions();
    let Some(active_idx) = app.input_handler.completion_idx() else {
        return;
    };

    let available_height = f.area().bottom().saturating_sub(input_area.bottom());
    let visible_count = completions
        .len()
        .min(6)
        .min(usize::from(available_height.saturating_sub(2)));
    if visible_count == 0 {
        return;
    }

    let start = active_idx
        .saturating_add(1)
        .saturating_sub(visible_count)
        .min(completions.len().saturating_sub(visible_count));
    let items = completions
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_count)
        .map(|(idx, path)| {
            let name = path
                .file_name()
                .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy());
            let text = format!("{name}/");
            if idx == active_idx {
                ListItem::new(text).style(Style::default().fg(Color::Black).bg(Color::Cyan))
            } else {
                ListItem::new(text)
            }
        })
        .collect::<Vec<_>>();

    let popup_area = Rect::new(
        input_area.x,
        input_area.bottom(),
        input_area.width,
        u16::try_from(visible_count + 2).unwrap_or(available_height),
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));
    let list = List::new(items).block(block);

    f.render_widget(Clear, popup_area);
    f.render_widget(list, popup_area);
}

fn render_filter_input(f: &mut Frame, app: &App) {
    let size = f.area();
    let width = size.width.min(60);
    let input_area =
        Rect::new((size.width.saturating_sub(width)) / 2, 1, width, 3).intersection(size);

    let filtered = app.filtered_torrents().len();
    let total = app.torrents.len();

    let block = Block::default()
        .title(" Search ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .title_alignment(Alignment::Left)
        .title_bottom(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                format!("{filtered}/{total}"),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" matches "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(Clear, input_area);
    f.render_widget(block, input_area);

    let text_area = input_area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    let prompt = Span::styled("> ", Style::default().fg(Color::Cyan).bold());
    let prompt_width = u16::try_from(prompt.width()).unwrap_or(u16::MAX);
    let view = input_view(
        &app.input_handler.text,
        app.input_handler.cursor_position,
        text_area.width.saturating_sub(prompt_width),
    );
    let input = Paragraph::new(Line::from(vec![prompt, Span::raw(view.visible)]));
    f.render_widget(input, text_area);

    if text_area.width > prompt_width && text_area.height > 0 {
        f.set_cursor_position(Position::new(
            text_area.x + prompt_width + view.cursor_column,
            text_area.y,
        ));
    }
}

/// The part of a single-line input that fits its box, with the cursor kept on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InputView<'a> {
    /// Text to draw from the left edge of the box.
    visible: &'a str,
    /// Cursor column relative to the left edge of the box.
    cursor_column: u16,
}

/// Scrolls `text` horizontally so the cursor at byte offset `cursor` fits in `width` display
/// columns.
///
/// Text is measured and cropped by whole grapheme clusters, with the cell width ratatui uses
/// when it draws a cluster, so emoji sequences and variation selectors are never split. A
/// `cursor` inside a cluster snaps to the start of that cluster, and one past the end of `text`
/// snaps to the end.
///
/// Scrolls only as far as needed, so text that fits stays left-aligned. The cells under the
/// cursor are kept visible, including a wide cluster at the cursor.
fn input_view(text: &str, cursor: usize, width: u16) -> InputView<'_> {
    let width = usize::from(width);
    let cursor = grapheme_start(text, cursor);
    let cursor_cell = text[cursor..]
        .graphemes(true)
        .next()
        .map_or(1, |grapheme| grapheme_width(grapheme).max(1));
    let prefix_budget = width.saturating_sub(cursor_cell);

    let mut start = cursor;
    let mut cursor_column = 0;
    for (index, grapheme) in text[..cursor].grapheme_indices(true).rev() {
        let grapheme_width = grapheme_width(grapheme);
        if cursor_column + grapheme_width > prefix_budget {
            break;
        }
        cursor_column += grapheme_width;
        start = index;
    }

    let mut end = start;
    let mut used = 0;
    for (index, grapheme) in text[start..].grapheme_indices(true) {
        let grapheme_width = grapheme_width(grapheme);
        if used + grapheme_width > width {
            break;
        }
        used += grapheme_width;
        end = start + index + grapheme.len();
    }

    InputView {
        visible: &text[start..end],
        cursor_column: u16::try_from(cursor_column).unwrap_or(u16::MAX),
    }
}

/// Start of the grapheme cluster containing byte offset `cursor`, or `text.len()` past the end.
fn grapheme_start(text: &str, cursor: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .chain(iter::once(text.len()))
        .take_while(|&index| index <= cursor)
        .last()
        .unwrap_or_default()
}

/// Cells ratatui uses to draw `grapheme`. It skips clusters containing control characters.
fn grapheme_width(grapheme: &str) -> usize {
    if grapheme.contains(char::is_control) {
        return 0;
    }
    usize::from(grapheme.cell_width())
}

/// Names listed in the delete dialog before collapsing the rest into a count.
const MAX_DELETE_NAMES: usize = 3;

fn render_confirm_delete(f: &mut Frame, target: &DeleteTarget) {
    let count = target.names().len();
    let subject = if count == 1 {
        "torrent".to_owned()
    } else {
        format!("{count} torrents")
    };
    let title = if target.delete_local_data() {
        format!(" Delete {subject} and local data? ")
    } else {
        format!(" Delete {subject}? ")
    };

    let mut lines = target
        .names()
        .take(MAX_DELETE_NAMES)
        .map(Line::from)
        .collect::<Vec<_>>();
    if count > MAX_DELETE_NAMES {
        lines.push(Line::from(format!("+{} more", count - MAX_DELETE_NAMES)));
    }
    lines.push(Line::from("[y]es / [n]o"));

    let content_width = lines
        .iter()
        .map(Line::width)
        .chain([title.len()])
        .max()
        .unwrap_or_default();
    let size = f.area();
    let dialog_width = u16::try_from(content_width + 4)
        .unwrap_or(u16::MAX)
        .max(40)
        .min(size.width);
    let dialog_height = u16::try_from(lines.len() + 2)
        .unwrap_or(u16::MAX)
        .min(size.height);
    let dialog_area = Rect::new(
        (size.width - dialog_width) / 2,
        (size.height - dialog_height) / 2,
        dialog_width,
        dialog_height,
    );

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::Red).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    f.render_widget(Clear, dialog_area);
    f.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(block),
        dialog_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(visible: &str, cursor_column: u16) -> InputView<'_> {
        InputView {
            visible,
            cursor_column,
        }
    }

    #[test]
    fn ascii_that_fits_is_not_scrolled() {
        assert_eq!(input_view("abc", 3, 10), view("abc", 3));
        assert_eq!(input_view("abc", 1, 10), view("abc", 1));
        assert_eq!(input_view("abc", 0, 10), view("abc", 0));
    }

    #[test]
    fn multibyte_cursor_uses_display_columns() {
        assert_eq!(input_view("äbc", "ä".len(), 10), view("äbc", 1));
        assert_eq!(input_view("äöü", "äöü".len(), 10), view("äöü", 3));
    }

    #[test]
    fn wide_characters_take_two_columns() {
        assert_eq!(input_view("日本語", "日本".len(), 10), view("日本語", 4));
        assert_eq!(input_view("日本語", "日本語".len(), 10), view("日本語", 6));
    }

    #[test]
    fn long_text_scrolls_to_keep_cursor_at_end_visible() {
        assert_eq!(input_view("abcdefghij", 10, 5), view("ghij", 4));
    }

    #[test]
    fn long_text_scrolls_for_cursor_in_the_middle() {
        assert_eq!(input_view("abcdefghij", 0, 5), view("abcde", 0));
        assert_eq!(input_view("abcdefghij", 3, 5), view("abcde", 3));
        assert_eq!(input_view("abcdefghij", 7, 5), view("defgh", 4));
    }

    #[test]
    fn long_wide_text_scrolls_by_whole_characters() {
        assert_eq!(
            input_view("日本語中文", "日本語中文".len(), 5),
            view("中文", 4)
        );
        assert_eq!(input_view("日本語", "日本語".len(), 4), view("語", 2));
    }

    #[test]
    fn wide_character_under_cursor_stays_visible() {
        assert_eq!(input_view("ab日", 2, 3), view("b日", 1));
    }

    #[test]
    fn zwj_emoji_is_measured_and_cropped_as_one_cluster() {
        let technologist = "👩\u{200d}💻";
        assert_eq!(
            input_view(technologist, technologist.len(), 10),
            view(technologist, 2)
        );
        assert_eq!(
            input_view(technologist, technologist.len(), 3),
            view(technologist, 2)
        );
        assert_eq!(input_view(technologist, technologist.len(), 2), view("", 0));
    }

    #[test]
    fn variation_selector_emoji_is_measured_as_one_cluster() {
        let heart = "❤\u{fe0f}";
        assert_eq!(input_view(heart, heart.len(), 10), view(heart, 2));
        assert_eq!(input_view("a❤\u{fe0f}", 1, 2), view(heart, 0));
    }

    #[test]
    fn cursor_inside_a_cluster_snaps_to_its_start() {
        let text = "a👩\u{200d}💻b";
        assert_eq!(input_view(text, "a👩".len(), 10), view(text, 1));
        assert_eq!(input_view(text, "a👩\u{200d}".len() + 1, 10), view(text, 1));
    }

    #[test]
    fn zero_width_box_shows_nothing() {
        assert_eq!(input_view("abc", 3, 0), view("", 0));
    }

    #[test]
    fn invalid_cursor_is_clamped_to_a_char_boundary() {
        assert_eq!(input_view("ä", 1, 5), view("ä", 0));
        assert_eq!(input_view("abc", 10, 5), view("abc", 3));
    }
}
