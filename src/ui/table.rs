use super::to_color;
use crate::{
    app::utils::Wrapper,
    config::{color::ColorConfig, time::TimeConfig},
};
use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style, Styled},
    widgets::{Block, BorderType, Borders, Row, Table},
};
use std::collections::HashSet;
use transmission_rpc::types::{Torrent, TorrentGetField, TorrentStatus};

pub fn build_table(
    torrents: &[&Torrent],
    selected: &HashSet<i64>,
    colors: &ColorConfig,
    time: &TimeConfig,
    fields: &[TorrentGetField],
) -> Table<'static> {
    let select_style = select_style(colors);
    let header_style = header_style(colors);
    let highlight_row_style = highlighted_row_style(colors);

    let rows = torrents
        .iter()
        .map(|t| make_row(t, fields, selected, select_style, colors, time))
        .collect::<Vec<_>>();

    let widths = fields
        .iter()
        .map(|&field| column_width(field))
        .collect::<Vec<_>>();

    let header = Row::new(fields.iter().map(|&field| field.title()))
        .style(header_style)
        .bottom_margin(1);

    Table::new(rows, widths)
        .block(default_block())
        .header(header)
        .row_highlight_style(highlight_row_style)
        .highlight_symbol("▶ ")
        .column_spacing(1)
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

fn select_style(cfg: &ColorConfig) -> Style {
    let fg = to_color(&cfg.highlight_foreground);
    let bg = to_color(&cfg.highlight_background);
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

fn make_row(
    torrent: &Torrent,
    fields: &[TorrentGetField],
    selected: &HashSet<i64>,
    highlight: Style,
    colors: &ColorConfig,
    time: &TimeConfig,
) -> Row<'static> {
    let status_style = status_style(torrent.status, colors);

    let cells = fields.iter().map(|&field| {
        if let Some(id) = torrent.id
            && selected.contains(&id)
        {
            return field.value(torrent, time).set_style(highlight);
        }
        field.value(torrent, time).set_style(status_style)
    });
    Row::new(cells)
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
    use super::column_width;
    use crate::app::utils::Wrapper;
    use ratatui::layout::Constraint;
    use transmission_rpc::types::TorrentGetField;

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
}
