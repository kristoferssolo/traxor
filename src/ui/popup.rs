use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn render_popup(r: Rect) -> Rect {
    let percent_y = 20;
    let popup_layput = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100 - percent_y),
            Constraint::Percentage(percent_y),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(0), Constraint::Percentage(100)])
        .split(popup_layput[1])[1]
}
