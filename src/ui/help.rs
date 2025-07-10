use crate::app::App;
use ratatui::{prelude::*, widgets::*};

pub fn render_help(frame: &mut Frame, app: &App) {
    let block = Block::default()
        .title("Help")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let keybinds = app.config.keybinds.clone();

    let rows = vec![
        Row::new(vec![
            Cell::from(keybinds.toggle_help),
            Cell::from("Show help"),
        ]),
        Row::new(vec![Cell::from(keybinds.quit), Cell::from("Quit")]),
        Row::new(vec![Cell::from(keybinds.prev_tab), Cell::from("Left")]),
        Row::new(vec![Cell::from(keybinds.next_tab), Cell::from("Right")]),
        Row::new(vec![Cell::from(keybinds.next_torrent), Cell::from("Down")]),
        Row::new(vec![Cell::from(keybinds.prev_torrent), Cell::from("Up")]),
        Row::new(vec![
            Cell::from(keybinds.switch_tab_1),
            Cell::from("Switch to All tab"),
        ]),
        Row::new(vec![
            Cell::from(keybinds.switch_tab_2),
            Cell::from("Switch to Active tab"),
        ]),
        Row::new(vec![
            Cell::from(keybinds.switch_tab_3),
            Cell::from("Switch to Downloading tab"),
        ]),
        Row::new(vec![
            Cell::from(keybinds.toggle_torrent),
            Cell::from("Toggle torrent"),
        ]),
        Row::new(vec![
            Cell::from(keybinds.toggle_all),
            Cell::from("Toggle all torrents"),
        ]),
        Row::new(vec![
            Cell::from(keybinds.delete),
            Cell::from("Delete torrent"),
        ]),
        Row::new(vec![
            Cell::from(keybinds.delete_force),
            Cell::from("Delete torrent and data"),
        ]),
        Row::new(vec![
            Cell::from(keybinds.select),
            Cell::from("Select torrent"),
        ]),
    ];

    let table = Table::new(
        rows,
        &[Constraint::Percentage(20), Constraint::Percentage(80)],
    )
    .block(block)
    .style(
        Style::default().fg(app
            .config
            .colors
            .get_color(&app.config.colors.info_foreground)),
    );

    let area = frame.area();
    let height = 15; // Desired height for the help menu
    let width = area.width; // Full width of the screen

    let popup_area = Rect::new(
        area.x + (area.width - width) / 2, // Center horizontally
        area.y + area.height - height,     // Position at the very bottom
        width,
        height,
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(table, popup_area);
}
