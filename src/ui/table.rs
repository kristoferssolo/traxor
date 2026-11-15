use crate::{
    app::utils::Wrapper,
    config::{color::ColorConfig, time::TimeConfig},
    ui::to_color,
};
use ratatui::{
    buffer::CellWidth,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Styled},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};
use std::{collections::HashSet, iter::repeat_n, str::FromStr};
use transmission_rpc::types::{Torrent, TorrentGetField, TorrentStatus};

/// Blank rows between the header and the first torrent row.
pub const HEADER_BOTTOM_MARGIN: u16 = 1;

/// Symbol drawn in front of the highlighted row.
const HIGHLIGHT_SYMBOL: &str = "▶ ";

/// Blank columns between adjacent table columns.
const COLUMN_SPACING: u16 = 1;

/// How the table places columns when their constraints leave width over.
const COLUMN_FLEX: Flex = Flex::Start;

/// Brightening of RGB progress tints on rows with their own background, as
/// `(numerator, denominator)`, so the highlight or selection stays visible inside the fill.
const ROW_BACKGROUND_TINT_SCALE: (u16, u16) = (13, 10);

/// Builds the torrent table for `area`.
///
/// `highlighted` is the [`TableState`](ratatui::widgets::TableState) selection the table will
/// be rendered with. It and `area` determine the Name column's rendered width, which the
/// progress fill needs.
pub fn build_table(
    torrents: &[&Torrent],
    selected: &HashSet<i64>,
    highlighted: Option<usize>,
    colors: &ColorConfig,
    time: &TimeConfig,
    fields: &[TorrentGetField],
    area: Rect,
) -> Table<'static> {
    let select_style = select_style(colors);
    let header_style = header_style(colors);
    let highlight_row_style = highlighted_row_style(colors);

    let widths = fields
        .iter()
        .map(|&field| column_width(field))
        .collect::<Vec<_>>();

    // `Table` clamps an out-of-range selection to the last row when rendering.
    let highlighted = highlighted.map(|idx| idx.min(torrents.len().saturating_sub(1)));
    let rendered_widths = rendered_column_widths(
        &widths,
        default_block().inner(area).width,
        highlighted.is_some(),
    );

    let rows = torrents
        .iter()
        .enumerate()
        .map(|(idx, torrent)| {
            let background = if highlighted == Some(idx) {
                Some(highlight_row_style)
            } else if torrent.id.is_some_and(|id| selected.contains(&id)) {
                Some(select_style)
            } else {
                None
            };
            make_row(torrent, fields, &rendered_widths, background, colors, time)
        })
        .collect::<Vec<_>>();

    let header = Row::new(fields.iter().map(|&field| field.title()))
        .style(header_style)
        .bottom_margin(HEADER_BOTTOM_MARGIN);

    Table::new(rows, widths)
        .block(default_block())
        .header(header)
        // The highlighted row gets its background from its row style, underneath the
        // progress fill, so the table only re-applies the foreground and modifiers on top.
        .row_highlight_style(Style {
            bg: None,
            ..highlight_row_style
        })
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
        _ => Constraint::Length(field.width()),
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

fn highlighted_row_style(cfg: &ColorConfig) -> Style {
    let fg = to_color(&cfg.highlight_foreground);
    let bg = to_color(&cfg.highlight_background);
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
}

/// Builds one torrent row. `widths` are the rendered column widths and `background` the
/// row's own style, if it is highlighted or selected.
fn make_row(
    torrent: &Torrent,
    fields: &[TorrentGetField],
    widths: &[u16],
    background: Option<Style>,
    colors: &ColorConfig,
    time: &TimeConfig,
) -> Row<'static> {
    let status_style = status_style(torrent.status, colors);
    let tint = progress_color(torrent.status, colors).map(|color| {
        if background.is_some() {
            brighten(color)
        } else {
            color
        }
    });
    let percent_done = torrent.percent_done.unwrap_or_default();

    let cells = fields.iter().zip(widths).map(|(&field, &width)| {
        let value = field.value(torrent, time);
        match (field, tint) {
            (TorrentGetField::Name, Some(tint)) => {
                name_cell(&value, width, percent_done, tint).style(status_style)
            }
            _ => Cell::from(value.set_style(status_style)),
        }
    });
    let row = Row::new(cells);

    match background {
        Some(style) => row.style(style),
        None => row,
    }
}

/// Builds a Name cell whose background is `tint` across `percent_done` of its `width`.
fn name_cell(name: &str, width: u16, percent_done: f32, tint: Color) -> Cell<'static> {
    let (filled, unfilled) = split_progress(name, width, percent_done);
    Cell::from(Line::from(vec![
        Span::styled(filled, Style::default().bg(tint)),
        Span::raw(unfilled),
    ]))
}

/// Fits `text` to exactly `width` display columns and splits it into the part covered by a
/// `ratio` progress fill and the rest.
///
/// Graphemes are filtered and measured as `Buffer::set_stringn` renders them. Text that
/// overflows `width` is cut before the first grapheme that does not fit, and the remainder is
/// padded with spaces so the fill also covers blank columns. A grapheme belongs to the filled
/// part when its midpoint is inside the fill, so wide characters are never split; a wide
/// character straddling the boundary rounds the fill up.
fn split_progress(text: &str, width: u16, ratio: f32) -> (String, String) {
    let filled_width = usize::from(filled_columns(width, ratio));
    let width = usize::from(width);
    let mut filled = String::new();
    let mut unfilled = String::new();
    let mut column = 0;

    let span = Span::raw(text);
    for grapheme in span.styled_graphemes(Style::default()) {
        let grapheme_width = usize::from(grapheme.symbol.cell_width());
        if column + grapheme_width > width {
            break;
        }
        let part = if 2 * column + grapheme_width <= 2 * filled_width {
            &mut filled
        } else {
            &mut unfilled
        };
        part.push_str(grapheme.symbol);
        column += grapheme_width;
    }

    let filled_padding = filled_width.saturating_sub(column);
    filled.extend(repeat_n(' ', filled_padding));
    unfilled.extend(repeat_n(' ', width - column - filled_padding));
    (filled, unfilled)
}

/// Returns how many of `width` columns a `ratio` progress fill covers, rounded to the
/// nearest column. Ratios outside `0.0..=1.0` are clamped and `NaN` fills nothing.
fn filled_columns(width: u16, ratio: f32) -> u16 {
    if ratio.is_nan() {
        return 0;
    }
    let filled = (ratio.clamp(0.0, 1.0) * f32::from(width)).round();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "`filled` is a whole number in `0..=width`"
    )]
    let filled = filled as u16;
    filled
}

/// Scales an RGB color by [`ROW_BACKGROUND_TINT_SCALE`]. Named and indexed colors have no
/// brighter variant to derive, so they are returned unchanged.
fn brighten(color: Color) -> Color {
    let Color::Rgb(red, green, blue) = color else {
        return color;
    };
    let (numerator, denominator) = ROW_BACKGROUND_TINT_SCALE;
    let scale = |channel: u8| {
        let scaled = (u16::from(channel) * numerator + denominator / 2) / denominator;
        u8::try_from(scaled).unwrap_or(u8::MAX)
    };
    Color::Rgb(scale(red), scale(green), scale(blue))
}

/// Returns the progress fill color for `status`, or `None` when the status is unknown or the
/// configured color is invalid, in which case the Name cell has no fill.
fn progress_color(status: Option<TorrentStatus>, colors: &ColorConfig) -> Option<Color> {
    let color = match status? {
        TorrentStatus::Downloading => &colors.progress_downloading,
        TorrentStatus::Seeding => &colors.progress_seeding,
        TorrentStatus::Stopped => &colors.progress_stopped,
        TorrentStatus::Verifying => &colors.progress_verifying,
        TorrentStatus::QueuedToDownload
        | TorrentStatus::QueuedToSeed
        | TorrentStatus::QueuedToVerify => &colors.progress_queued,
    };
    Color::from_str(color).ok()
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

    const DOWNLOADING_TINT: Color = Color::Rgb(0x16, 0x3a, 0x40);

    fn default_config() -> Config {
        assert_ok!(toml::from_str(include_str!("../../config/default.toml")))
    }

    fn downloading_torrent(percent_done: f32) -> Torrent {
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
            &config.colors,
            &config.time,
            &fields,
            area,
        );
        let mut buf = Buffer::empty(area);
        let mut state = TableState::default().with_selected(highlighted);
        StatefulWidget::render(table, area, &mut buf, &mut state);
        buf
    }

    fn row_backgrounds(buf: &Buffer, columns: std::ops::Range<u16>) -> Vec<Color> {
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
    fn compact_columns_remain_fixed_width() {
        assert_eq!(
            column_width(TorrentGetField::Status),
            Constraint::Length(TorrentGetField::Status.width())
        );
    }

    #[test]
    fn split_empty_progress_pads_unfilled_part() {
        assert_eq!(
            split_progress("ubuntu", 10, 0.0),
            (String::new(), "ubuntu    ".to_owned())
        );
    }

    #[test]
    fn split_full_progress_pads_filled_part() {
        assert_eq!(
            split_progress("ubuntu", 10, 1.0),
            ("ubuntu    ".to_owned(), String::new())
        );
    }

    #[test]
    fn split_partial_progress_rounds_to_nearest_column() {
        assert_eq!(
            split_progress("ubuntu", 10, 0.5),
            ("ubunt".to_owned(), "u    ".to_owned())
        );
        // 0.34 * 10 = 3.4 rounds down, 0.36 * 10 = 3.6 rounds up.
        assert_eq!(split_progress("ubuntu", 10, 0.34).0, "ubu");
        assert_eq!(split_progress("ubuntu", 10, 0.36).0, "ubun");
        // The fill can extend past the end of the text.
        assert_eq!(
            split_progress("ab", 6, 0.5),
            ("ab ".to_owned(), "   ".to_owned())
        );
    }

    #[test]
    fn split_never_divides_wide_characters() {
        // Each character is two columns wide. A fill of 4 columns ends between characters.
        assert_eq!(
            split_progress("日本語", 8, 0.5),
            ("日本".to_owned(), "語  ".to_owned())
        );
        // A fill of 3 columns ends inside "本", which rounds the fill up to 4 columns.
        assert_eq!(
            split_progress("日本語", 10, 0.3),
            ("日本".to_owned(), "語    ".to_owned())
        );
    }

    #[test]
    fn split_truncates_text_longer_than_width() {
        assert_eq!(
            split_progress("abcdefghij", 4, 1.0),
            ("abcd".to_owned(), String::new())
        );
        assert_eq!(
            split_progress("abcdefghij", 4, 0.5),
            ("ab".to_owned(), "cd".to_owned())
        );
        // A wide character that would cross the cell edge is replaced by padding.
        assert_eq!(
            split_progress("ab日", 3, 0.0),
            (String::new(), "ab ".to_owned())
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

    #[test]
    fn brighten_scales_rgb_and_keeps_named_colors() {
        assert_eq!(brighten(DOWNLOADING_TINT), Color::Rgb(0x1d, 0x4b, 0x53));
        assert_eq!(
            brighten(Color::Rgb(0xff, 0x00, 0xc8)),
            Color::Rgb(0xff, 0x00, 0xff)
        );
        assert_eq!(brighten(Color::DarkGray), Color::DarkGray);
    }

    #[test]
    fn name_cell_background_follows_progress() {
        let buf = render(&downloading_torrent(0.4), &HashSet::new(), None);

        // Inner width 38 at x = 1: `%` is 10 wide, then a space, so Name spans x 12..39
        // (27 columns), and 40% of 27 rounds to 11 filled columns.
        let mut expected = vec![Color::Reset];
        expected.extend([DOWNLOADING_TINT; 11]);
        expected.extend([Color::Reset; 16]);
        assert_eq!(row_backgrounds(&buf, 11..39), expected);
    }

    #[test]
    fn highlighted_row_brightens_fill_and_keeps_highlight_elsewhere() {
        let buf = render(&downloading_torrent(0.4), &HashSet::new(), Some(0));
        let highlight = to_color(&default_config().colors.highlight_background);

        // The highlight symbol takes 2 columns, so Name spans x 14..39 (25 columns), and 40%
        // of 25 is 10 filled columns.
        let mut expected = vec![highlight];
        expected.extend([brighten(DOWNLOADING_TINT); 10]);
        expected.extend([highlight; 15]);
        assert_eq!(row_backgrounds(&buf, 13..39), expected);
    }

    #[test]
    fn selected_row_brightens_fill_and_keeps_selection_elsewhere() {
        let buf = render(&downloading_torrent(1.0), &HashSet::from([1]), None);
        let selection = to_color(&default_config().colors.selected_background);

        let mut expected = vec![selection];
        expected.extend([brighten(DOWNLOADING_TINT); 27]);
        assert_eq!(row_backgrounds(&buf, 11..39), expected);
    }
}
