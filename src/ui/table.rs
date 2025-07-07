use crate::app::{utils::Wrapper, App, Tab};
use ratatui::{
    layout::Constraint,
    style::{Style, Styled},
    widgets::{Block, BorderType, Borders, Row, Table},
};

pub fn render_table<'a>(app: &mut App, tab: Tab) -> Table<'a> {
    let fields = tab.fields();
    let selected = &app.torrents.selected.clone();
    let torrents = &app.torrents.set_fields(None).torrents;

    let highlight_bg = app.config.colors.get_color(&app.config.colors.highlight_background);
    let highlight_fg = app.config.colors.get_color(&app.config.colors.highlight_foreground);
    let highlight_style = Style::default().bg(highlight_bg).fg(highlight_fg);

    let rows: Vec<Row<'_>> = torrents
        .iter()
        .map(|torrent| {
            Row::new(
                fields
                    .iter()
                    .map(|&field| {
                        if let Some(id) = torrent.id {
                            if selected.contains(&id) {
                                return field.value(torrent).set_style(highlight_style);
                            }
                        }
                        field.value(torrent).into()
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    let widths = fields
        .iter()
        .map(|&field| Constraint::Length(field.width()))
        .collect::<Vec<_>>();

    let header_fg = app.config.colors.get_color(&app.config.colors.warning_foreground);
    let header = Row::new(
        fields
            .iter()
            .map(|&field| field.title())
            .collect::<Vec<_>>(),
    )
    .style(Style::default().fg(header_fg));

    let row_highlight_bg = app.config.colors.get_color(&app.config.colors.info_foreground);
    let row_highlight_fg = app.config.colors.get_color(&app.config.colors.highlight_foreground);
    let row_highlight_style = Style::default().bg(row_highlight_bg).fg(row_highlight_fg);

    Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .header(header)
        .row_highlight_style(row_highlight_style)
        .column_spacing(1)
}
