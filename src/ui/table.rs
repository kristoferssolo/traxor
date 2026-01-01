use super::to_color;
use crate::{app::utils::Wrapper, config::color::ColorConfig};
use ratatui::{
    layout::Constraint,
    style::{Style, Styled},
    widgets::{Block, BorderType, Borders, Row, Table},
};
use std::collections::HashSet;
use transmission_rpc::types::{Torrent, TorrentGetField, TorrentStatus};

pub fn build_table(
    torrents: &[&Torrent],
    selected: &HashSet<i64>,
    colors: &ColorConfig,
    fields: &[TorrentGetField],
) -> Table<'static> {
    let row_style = row_style(colors);
    let header_style = header_style(colors);
    let highlight_row_style = hightlighted_row_style(colors);

    let rows = torrents
        .iter()
        .map(|t| make_row(t, fields, selected, row_style, colors))
        .collect::<Vec<_>>();

    let widths = fields
        .iter()
        .map(|&f| Constraint::Length(f.width()))
        .collect::<Vec<_>>();

    let header = Row::new(fields.iter().map(|&field| field.title())).style(header_style);

    Table::new(rows, widths)
        .block(default_block())
        .header(header)
        .row_highlight_style(highlight_row_style)
        .column_spacing(1)
}

fn default_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
}

fn row_style(cfg: &ColorConfig) -> Style {
    let fg = to_color(&cfg.highlight_foreground);
    let bg = to_color(&cfg.info_foreground);
    Style::default().bg(bg).fg(fg)
}

fn header_style(cfg: &ColorConfig) -> Style {
    let fg = to_color(&cfg.header_foreground);
    Style::default().fg(fg)
}

fn hightlighted_row_style(cfg: &ColorConfig) -> Style {
    let fg = to_color(&cfg.info_foreground);
    let bg = to_color(&cfg.highlight_foreground);
    Style::default().bg(bg).fg(fg)
}

fn make_row(
    torrent: &Torrent,
    fields: &[TorrentGetField],
    selected: &HashSet<i64>,
    highlight: Style,
    colors: &ColorConfig,
) -> Row<'static> {
    let status_style = status_style(torrent.status, colors);

    let cells = fields.iter().map(|&field| {
        if let Some(id) = torrent.id
            && selected.contains(&id)
        {
            return field.value(torrent).set_style(highlight);
        }
        field.value(torrent).set_style(status_style)
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
