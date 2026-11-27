use crate::{
    app::{sort::Sort, utils::Wrapper},
    config::{Config, color::ColorConfig, time::TimeConfig},
    ui::to_color,
};
use ratatui::{
    buffer::CellWidth,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Styled},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};
use std::{collections::HashSet, iter::repeat_n};
use transmission_rpc::types::{Torrent, TorrentGetField, TorrentStatus};

/// Blank rows between the header and the first torrent row.
pub const HEADER_BOTTOM_MARGIN: u16 = 1;

/// Symbol drawn in front of the highlighted row.
const HIGHLIGHT_SYMBOL: &str = "▶ ";

/// Blank columns between adjacent table columns.
const COLUMN_SPACING: u16 = 1;

/// How the table places columns when their constraints leave width over.
const COLUMN_FLEX: Flex = Flex::Start;

/// Cells a sort indicator such as ` ▼` adds to a column title.
const SORT_INDICATOR_WIDTH: u16 = 2;

/// Builds the torrent table for `area`.
///
/// `highlighted` is the [`TableState`](ratatui::widgets::TableState) selection the table will
/// be rendered with. It and `area` determine the Name column's rendered width, which the
/// progress underline needs.
///
/// `sort` marks the sorted column's header with its direction.
pub fn build_table(
    torrents: &[&Torrent],
    selected: &HashSet<i64>,
    highlighted: Option<usize>,
    config: &Config,
    fields: &[TorrentGetField],
    area: Rect,
    sort: Option<Sort>,
) -> Table<'static> {
    let colors = &config.colors;
    let time = &config.time;
    let select_style = select_style(colors);
    let header_style = header_style(colors);
    let sorted_header_style = sorted_header_style(colors);
    let highlight_row_style = highlighted_row_style(colors);

    let widths = fields
        .iter()
        .map(|&field| column_width(field))
        .collect::<Vec<_>>();

    let rendered_widths = rendered_column_widths(
        &widths,
        default_block().inner(area).width,
        highlighted.is_some(),
    );

    let rows = torrents
        .iter()
        .map(|torrent| {
            let background = torrent
                .id
                .is_some_and(|id| selected.contains(&id))
                .then_some(select_style);
            make_row(torrent, fields, &rendered_widths, background, colors, time)
        })
        .collect::<Vec<_>>();

    let header_cells = fields.iter().enumerate().map(|(column, &field)| {
        let title = field.title();
        match sort {
            Some(sort) if sort.column == column => {
                Cell::from(format!("{title} {}", sort.direction.arrow())).style(sorted_header_style)
            }
            _ => Cell::from(title),
        }
    });
    let header = Row::new(header_cells)
        .style(header_style)
        .bottom_margin(HEADER_BOTTOM_MARGIN);

    Table::new(rows, widths)
        .block(default_block())
        .header(header)
        .row_highlight_style(highlight_row_style)
        .highlight_symbol(HIGHLIGHT_SYMBOL)
        .column_spacing(COLUMN_SPACING)
        .flex(COLUMN_FLEX)
}

fn default_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
}

fn column_width(field: TorrentGetField) -> Constraint {
    match field {
        // Keep the torrent name flexible so wide terminals show more of the thing users
        // actually scan, while narrower terminals can still render compact metadata columns.
        TorrentGetField::Name => Constraint::Fill(1),
        TorrentGetField::DownloadDir | TorrentGetField::TrackerList => Constraint::Min(20),
        TorrentGetField::HashString => Constraint::Min(16),
        // Leave room for the sort indicator so sorting never truncates the title.
        _ => {
            let title_width = u16::try_from(Span::raw(field.title()).width()).unwrap_or(u16::MAX);
            Constraint::Length(
                field
                    .width()
                    .max(title_width.saturating_add(SORT_INDICATOR_WIDTH)),
            )
        }
    }
}

/// Returns the rendered width of each column in a table whose inner area is `inner_width`
/// wide, resolved the same way as ratatui's `Table::get_column_widths`: the highlight symbol
/// column is reserved first when a row is highlighted, then `constraints` split the rest.
fn rendered_column_widths(
    constraints: &[Constraint],
    inner_width: u16,
    has_highlight: bool,
) -> Vec<u16> {
    let selection_width = if has_highlight {
        u16::try_from(Span::raw(HIGHLIGHT_SYMBOL).width()).unwrap_or(u16::MAX)
    } else {
        0
    };
    let [_, columns_area] =
        Layout::horizontal([Constraint::Length(selection_width), Constraint::Fill(0)])
            .areas(Rect::new(0, 0, inner_width, 1));
    Layout::horizontal(constraints.iter().copied())
        .flex(COLUMN_FLEX)
        .spacing(COLUMN_SPACING)
        .split(columns_area)
        .iter()
        .map(|column| column.width)
        .collect()
}

fn select_style(cfg: &ColorConfig) -> Style {
    let fg = to_color(&cfg.selected_foreground);
    let bg = to_color(&cfg.selected_background);
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
}

fn header_style(cfg: &ColorConfig) -> Style {
    let fg = to_color(&cfg.header_foreground);
    Style::default()
        .fg(fg)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

/// Header cell of the sorted column, inverted from the other headers.
fn sorted_header_style(cfg: &ColorConfig) -> Style {
    let bg = to_color(&cfg.header_foreground);
    Style::default()
        .fg(Color::Black)
        .bg(bg)
        .add_modifier(Modifier::BOLD)
        .remove_modifier(Modifier::UNDERLINED)
}

fn highlighted_row_style(cfg: &ColorConfig) -> Style {
    let fg = to_color(&cfg.highlight_foreground);
    let bg = to_color(&cfg.highlight_background);
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
}

/// Builds one torrent row. `widths` are the rendered column widths and `background` the
/// row's own style, if it is selected. The table applies the highlight on top.
fn make_row(
    torrent: &Torrent,
    fields: &[TorrentGetField],
    widths: &[u16],
    background: Option<Style>,
    colors: &ColorConfig,
    time: &TimeConfig,
) -> Row<'static> {
    let status_style = status_style(torrent.status, colors);
    let percent_done = torrent.percent_done.unwrap_or_default();

    let cells = fields.iter().zip(widths).map(|(&field, &width)| {
        let value = field.value(torrent, time);
        match field {
            TorrentGetField::Name => name_cell(&value, width, percent_done).style(status_style),
            _ => Cell::from(value.set_style(status_style)),
        }
    });
    let row = Row::new(cells);

    match background {
        Some(style) => row.style(style),
        None => row,
    }
}

/// Builds a Name cell that underlines `percent_done` of the column.
///
/// The underline takes the text color and draws no background, so the terminal's own
/// background, including transparency, shows through.
fn name_cell(name: &str, width: u16, percent_done: f32) -> Cell<'static> {
    let (done, rest) = split_progress(name, width, percent_done);
    Cell::from(Line::from(vec![
        Span::styled(done, Style::default().add_modifier(Modifier::UNDERLINED)),
        Span::raw(rest),
    ]))
}

/// Cuts `text` to at most `width` display columns, pads it with spaces to exactly `width`,
/// and splits it into the part covered by a `ratio` progress and the rest.
///
/// Progress is measured against the whole column, so equal progress ends at the same column
/// on every row, however long the name. Graphemes are filtered and measured as
/// `Buffer::set_stringn` renders them, and text that overflows `width` is cut before the
/// first grapheme that does not fit. A grapheme belongs to the covered part when its
/// midpoint is inside it, so wide characters are never split; a wide character straddling
/// the boundary rounds the covered part up.
fn split_progress(text: &str, width: u16, ratio: f32) -> (String, String) {
    let covered_width = usize::from(filled_columns(width, ratio));
    let width = usize::from(width);
    let span = Span::raw(text);
    let mut done = String::new();
    let mut rest = String::new();
    let mut column: usize = 0;
    for grapheme in span.styled_graphemes(Style::default()) {
        let grapheme_width = usize::from(grapheme.symbol.cell_width());
        let next_column = column.saturating_add(grapheme_width);
        if next_column > width {
            break;
        }
        // A grapheme belongs to `done` when its midpoint is within the covered columns
        let midpoint_twice = column.saturating_mul(2).saturating_add(grapheme_width);
        let part = if midpoint_twice <= covered_width.saturating_mul(2) {
            &mut done
        } else {
            &mut rest
        };
        part.push_str(grapheme.symbol);
        column = next_column;
    }

    // `rest` is empty whenever the text ends before the boundary, so padding `done` first
    // keeps the parts in order.
    let done_padding = covered_width.saturating_sub(column);
    done.extend(repeat_n(' ', done_padding));
    rest.extend(repeat_n(
        ' ',
        width.saturating_sub(column).saturating_sub(done_padding),
    ));
    (done, rest)
}

/// Returns how many of `width` columns a `ratio` progress covers, rounded to the nearest
/// column. Ratios outside `0.0..=1.0` are clamped and `NaN` covers nothing.
fn filled_columns(width: u16, ratio: f32) -> u16 {
    if ratio.is_nan() {
        return 0;
    }
    let filled = (ratio.clamp(0.0, 1.0) * f32::from(width)).round();
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "`filled` is a whole number in `0..=width`"
    )]
    let filled = filled as u16;
    filled
}

fn status_style(status: Option<TorrentStatus>, colors: &ColorConfig) -> Style {
    let color = match status {
        Some(TorrentStatus::Downloading) => &colors.status_downloading,
        Some(TorrentStatus::Seeding) => &colors.status_seeding,
        Some(TorrentStatus::Stopped) => &colors.status_stopped,
        Some(TorrentStatus::Verifying) => &colors.status_verifying,
        Some(
            TorrentStatus::QueuedToDownload
            | TorrentStatus::QueuedToSeed
            | TorrentStatus::QueuedToVerify,
        ) => &colors.status_queued,
        None => &colors.info_foreground,
    };
    Style::default().fg(to_color(color))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use claims::assert_ok;
    use ratatui::{
        buffer::Buffer,
        widgets::{StatefulWidget, TableState},
    };
    use std::ops::Range;

    fn default_config() -> Config {
        assert_ok!(toml::from_str(include_str!("../../config/default.toml")))
    }

    fn downloading_torrent(percent_done: f32) -> Torrent {
        #[expect(clippy::as_conversions, reason = "reads the enum's wire value")]
        let status = TorrentStatus::Downloading as u8;
        assert_ok!(toml::from_str(&format!(
            "id = 1\nname = \"ubuntu.iso\"\nstatus = {status}\npercentDone = {percent_done:?}"
        )))
    }

    /// Renders a 40-column table with a `%` column followed by the Name column and returns
    /// the buffer. The first torrent row is at y = 3, below the border, header, and margin.
    fn render(torrent: &Torrent, selected: &HashSet<i64>, highlighted: Option<usize>) -> Buffer {
        let config = default_config();
        let fields = [TorrentGetField::PercentDone, TorrentGetField::Name];
        let area = Rect::new(0, 0, 40, 5);
        let table = build_table(
            &[torrent],
            selected,
            highlighted,
            &config,
            &fields,
            area,
            None,
        );
        let mut buf = Buffer::empty(area);
        let mut state = TableState::default().with_selected(highlighted);
        StatefulWidget::render(table, area, &mut buf, &mut state);
        buf
    }

    fn row_backgrounds(buf: &Buffer, columns: Range<u16>) -> Vec<Color> {
        columns.map(|x| buf[(x, 3)].bg).collect()
    }

    #[test]
    fn name_column_uses_fill() {
        assert_eq!(column_width(TorrentGetField::Name), Constraint::Fill(1));
    }

    #[test]
    fn long_text_columns_use_minimum_widths() {
        assert_eq!(
            column_width(TorrentGetField::DownloadDir),
            Constraint::Min(20)
        );
        assert_eq!(
            column_width(TorrentGetField::TrackerList),
            Constraint::Min(20)
        );
        assert_eq!(
            column_width(TorrentGetField::HashString),
            Constraint::Min(16)
        );
    }

    #[test]
    fn fixed_columns_fit_title_and_sort_indicator() {
        // "Downloaded Ever" is as wide as its 15-cell column.
        assert_eq!(
            column_width(TorrentGetField::DownloadedEver),
            Constraint::Length(17)
        );
    }

    #[test]
    fn compact_columns_remain_fixed_width() {
        assert_eq!(
            column_width(TorrentGetField::Status),
            Constraint::Length(TorrentGetField::Status.width())
        );
    }

    #[test]
    fn split_measures_progress_against_the_column() {
        assert_eq!(
            split_progress("ubuntu", 10, 0.0),
            (String::new(), "ubuntu    ".to_owned())
        );
        assert_eq!(
            split_progress("ubuntu", 10, 1.0),
            ("ubuntu    ".to_owned(), String::new())
        );
        assert_eq!(
            split_progress("ubuntu", 10, 0.5),
            ("ubunt".to_owned(), "u    ".to_owned())
        );
        assert_eq!(
            split_progress("ubuntu", 10, 0.8),
            ("ubuntu  ".to_owned(), "  ".to_owned())
        );
        // 0.24 * 10 = 2.4 rounds down, 0.36 * 10 = 3.6 rounds up.
        assert_eq!(split_progress("ubuntu", 10, 0.24).0, "ub");
        assert_eq!(split_progress("ubuntu", 10, 0.36).0, "ubun");
    }

    #[test]
    fn split_never_divides_wide_characters() {
        // Each character is two columns wide. Half of 6 columns ends inside "本", which
        // rounds the covered part up to 4 columns.
        assert_eq!(
            split_progress("日本語", 6, 0.5),
            ("日本".to_owned(), "語".to_owned())
        );
        assert_eq!(
            split_progress("日本語", 6, 0.3),
            ("日".to_owned(), "本語".to_owned())
        );
    }

    #[test]
    fn split_cuts_text_longer_than_width() {
        assert_eq!(
            split_progress("abcdefghij", 4, 1.0),
            ("abcd".to_owned(), String::new())
        );
        assert_eq!(
            split_progress("abcdefghij", 4, 0.5),
            ("ab".to_owned(), "cd".to_owned())
        );
        // A wide character that would cross the cell edge is dropped and its column padded.
        assert_eq!(
            split_progress("ab日", 3, 1.0),
            ("ab ".to_owned(), String::new())
        );
    }

    #[test]
    fn split_zero_width_is_empty() {
        assert_eq!(
            split_progress("ubuntu", 0, 0.5),
            (String::new(), String::new())
        );
    }

    #[test]
    fn split_clamps_out_of_range_ratio() {
        assert_eq!(split_progress("ab", 2, 1.5).0, "ab");
        assert_eq!(split_progress("ab", 2, -0.5).1, "ab");
        assert_eq!(split_progress("ab", 2, f32::NAN).1, "ab");
    }

    fn underlined(buf: &Buffer, columns: Range<u16>) -> Vec<bool> {
        columns
            .map(|x| buf[(x, 3)].modifier.contains(Modifier::UNDERLINED))
            .collect()
    }

    #[test]
    fn name_underline_follows_progress_without_background() {
        let buf = render(&downloading_torrent(0.5), &HashSet::new(), None);

        // Inner width 38 at x = 1: `%` is 10 wide, then a space, so Name spans x 12..39.
        // 50% of its 27 columns rounds to 14, past the end of "ubuntu.iso".
        let mut expected = vec![true; 14];
        expected.extend([false; 13]);
        assert_eq!(underlined(&buf, 12..39), expected);
        assert_eq!(row_backgrounds(&buf, 11..39), vec![Color::Reset; 28]);

        let buf = render(&downloading_torrent(1.0), &HashSet::new(), None);
        assert_eq!(underlined(&buf, 12..39), vec![true; 27]);
    }

    #[test]
    fn highlighted_and_selected_rows_keep_their_background_across_the_name() {
        let config = default_config();

        let buf = render(&downloading_torrent(0.5), &HashSet::new(), Some(0));
        let highlight = to_color(&config.colors.highlight_background);
        // The highlight symbol shifts Name to x 14..39, and 50% of 25 columns rounds to 13.
        assert_eq!(row_backgrounds(&buf, 13..39), vec![highlight; 26]);
        let mut expected = vec![true; 13];
        expected.extend([false; 12]);
        assert_eq!(underlined(&buf, 14..39), expected);

        let buf = render(&downloading_torrent(1.0), &HashSet::from([1]), None);
        let selection = to_color(&config.colors.selected_background);
        assert_eq!(row_backgrounds(&buf, 11..39), vec![selection; 28]);
    }
}
