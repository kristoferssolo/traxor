use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn render_help(frame: &mut Frame, app: &App) {
    let block = Block::default()
        .title("Help")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let keybinds = &app.config.keybinds;

    let rows = vec![
        Row::new(vec![Cell::from(keybinds.toggle_help.as_deref().unwrap_or("?")), Cell::from("Show help")]),
        Row::new(vec![Cell::from(keybinds.quit.as_deref().unwrap_or("q")), Cell::from("Quit")]),
        Row::new(vec![Cell::from(keybinds.prev_tab.as_deref().unwrap_or("h")), Cell::from("Left")]),
        Row::new(vec![Cell::from(keybinds.next_tab.as_deref().unwrap_or("l")), Cell::from("Right")]),
        Row::new(vec![Cell::from(keybinds.next_torrent.as_deref().unwrap_or("j")), Cell::from("Down")]),
        Row::new(vec![Cell::from(keybinds.prev_torrent.as_deref().unwrap_or("k")), Cell::from("Up")]),
        Row::new(vec![Cell::from(keybinds.switch_tab_1.as_deref().unwrap_or("1")), Cell::from("Switch to All tab")]),
        Row::new(vec![Cell::from(keybinds.switch_tab_2.as_deref().unwrap_or("2")), Cell::from("Switch to Active tab")]),
        Row::new(vec![
            Cell::from(keybinds.switch_tab_3.as_deref().unwrap_or("3")),
            Cell::from("Switch to Downloading tab"),
        ]),
        Row::new(vec![Cell::from(keybinds.toggle_torrent.as_deref().unwrap_or("t")), Cell::from("Toggle torrent")]),
        Row::new(vec![Cell::from(keybinds.toggle_all.as_deref().unwrap_or("a")), Cell::from("Toggle all torrents")]),
        Row::new(vec![Cell::from(keybinds.delete.as_deref().unwrap_or("d")), Cell::from("Delete torrent")]),
        Row::new(vec![Cell::from(keybinds.delete_force.as_deref().unwrap_or("D")), Cell::from("Delete torrent and data")]),
        Row::new(vec![Cell::from(keybinds.select.as_deref().unwrap_or(" ")), Cell::from("Select torrent")]),
    ];

    let table = Table::new(
        rows,
        &[Constraint::Percentage(20), Constraint::Percentage(80)],
    )
    .block(block)
    .style(Style::default().fg(app.config.colors.get_color(&app.config.colors.info_foreground)));

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

