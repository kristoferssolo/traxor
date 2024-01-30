use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Row, Table},
};

use crate::app::{utils::Wrapper, App, Tab};

pub fn render_table<'a>(app: &mut App, tab: Tab) -> Table<'a> {
    let fields = tab.fields();
    let torrents = app.torrents.set_fields(None).torrents();

    let rows: Vec<Row<'_>> = torrents
        .iter()
        .map(|torrent| {
            Row::new(
                fields
                    .iter()
                    .map(|&field| field.value(torrent.clone()))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    let widths = fields
        .iter()
        .map(|&field| Constraint::Length(field.width()))
        .collect::<Vec<_>>();

    let header = Row::new(
        fields
            .iter()
            .map(|&field| field.title())
            .collect::<Vec<_>>(),
    )
    .style(Style::default().fg(Color::Yellow));
    Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .header(header)
        .highlight_style(Style::default().fg(Color::Red))
        .highlight_symbol(">> ")
        .column_spacing(1)
}
