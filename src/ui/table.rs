use ratatui::{
    layout::Constraint,
    style::{Color, Style, Styled},
    widgets::{Block, BorderType, Borders, Row, Table},
};

use crate::app::{utils::Wrapper, App, Tab};

pub fn render_table<'a>(app: &mut App, tab: Tab) -> Table<'a> {
    let fields = tab.fields();
    let selected = &app.torrents.selected.clone();
    let torrents = &app.torrents.set_fields(None).torrents;
    let highlight_style = Style::default().bg(Color::Magenta).fg(Color::Black);

    let rows: Vec<Row<'_>> = torrents
        .iter()
        .map(|torrent| {
            Row::new(
                fields
                    .iter()
                    .map(|&field| {
                        if let Some(id) = &torrent.clone().id {
                            if selected.contains(id) {
                                return field.value(torrent.clone()).set_style(highlight_style);
                            }
                        }
                        return field.value(torrent.clone()).into();
                    })
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

    let highlight_style = Style::default().bg(Color::Blue).fg(Color::Black);

    Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .header(header)
        .highlight_style(highlight_style)
        .column_spacing(1)
}
