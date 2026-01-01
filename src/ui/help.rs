use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, Clear, Row, Table},
};

pub fn render_help(frame: &mut Frame, app: &App) {
    let kb = &app.config.keybinds;
    let key_style = Style::default().fg(Color::Yellow).bold();
    let select_key = display_key(&kb.select);

    let rows = vec![
        section_row("── Navigation ──"),
        key_row(&kb.prev_torrent, "Move up", key_style),
        key_row(&kb.next_torrent, "Move down", key_style),
        key_row(&kb.prev_tab, "Previous tab", key_style),
        key_row(&kb.next_tab, "Next tab", key_style),
        key_row(&kb.switch_tab_1, "All torrents", key_style),
        key_row(&kb.switch_tab_2, "Active torrents", key_style),
        key_row(&kb.switch_tab_3, "Downloading", key_style),
        Row::default(),
        section_row("── Actions ──"),
        key_row(&kb.toggle_torrent, "Start/stop torrent", key_style),
        key_row(&kb.toggle_all, "Start/stop all", key_style),
        key_row(&select_key, "Multi-select", key_style),
        key_row(&kb.move_torrent, "Move torrent", key_style),
        key_row(&kb.rename_torrent, "Rename torrent", key_style),
        key_row(&kb.delete, "Remove torrent", key_style),
        key_row(&kb.delete_force, "Delete with data", key_style),
        Row::default(),
        section_row("── General ──"),
        key_row(&kb.toggle_help, "Toggle help", key_style),
        key_row(&kb.quit, "Quit", key_style),
    ];

    #[allow(clippy::cast_possible_truncation)]
    let height = rows.len() as u16 + 4;
    let width = 40;

    let block = Block::default()
        .title(" Keybindings ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let table = Table::new(rows, [Constraint::Length(16), Constraint::Fill(1)]).block(block);

    let area = frame.area();
    let popup_area = Rect::new(
        (area.width.saturating_sub(width)) / 2,
        (area.height.saturating_sub(height)) / 2,
        width.min(area.width),
        height.min(area.height),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(table, popup_area);
}

fn key_row<'a>(key: &'a str, desc: &'a str, key_style: Style) -> Row<'a> {
    Row::new(vec![
        Cell::from(format!("  {key}")).style(key_style),
        Cell::from(desc),
    ])
}

fn section_row(title: &str) -> Row<'_> {
    Row::new(vec![
        Cell::from(title).style(Style::default().fg(Color::DarkGray)),
    ])
}

fn display_key(key: &str) -> String {
    match key {
        " " => "Space".to_string(),
        k => k.to_string(),
    }
}
