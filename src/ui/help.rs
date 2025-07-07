use ratatui::{prelude::*, widgets::*};

pub fn render_help(frame: &mut Frame) {
    let block = Block::default()
        .title("Help")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let rows = vec![
        Row::new(vec![Cell::from("?"), Cell::from("Show help")]),
        Row::new(vec![Cell::from("q"), Cell::from("Quit")]),
        Row::new(vec![Cell::from("h"), Cell::from("Left")]),
        Row::new(vec![Cell::from("l"), Cell::from("Right")]),
        Row::new(vec![Cell::from("j"), Cell::from("Down")]),
        Row::new(vec![Cell::from("k"), Cell::from("Up")]),
        Row::new(vec![Cell::from("1"), Cell::from("Switch to All tab")]),
        Row::new(vec![Cell::from("2"), Cell::from("Switch to Active tab")]),
        Row::new(vec![
            Cell::from("3"),
            Cell::from("Switch to Downloading tab"),
        ]),
        Row::new(vec![Cell::from("t"), Cell::from("Toggle torrent")]),
        Row::new(vec![Cell::from("a"), Cell::from("Toggle all torrents")]),
        Row::new(vec![Cell::from("d"), Cell::from("Delete torrent")]),
        Row::new(vec![Cell::from("D"), Cell::from("Delete torrent and data")]),
        Row::new(vec![Cell::from(" "), Cell::from("Select torrent")]),
    ];

    let table = Table::new(
        rows,
        &[Constraint::Percentage(20), Constraint::Percentage(80)],
    )
    .block(block)
    .style(Style::default().fg(Color::Green));

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

