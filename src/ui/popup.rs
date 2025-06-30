use ratatui::layout::Rect;

pub fn render_popup(r: Rect) -> Rect {
    let vertical_margin = r.height / 5;
    let horizontal_margin = r.width / 5;

    Rect::new(
        r.x + horizontal_margin,
        r.y + vertical_margin,
        r.width - (2 * horizontal_margin),
        r.height - (2 * vertical_margin),
    )
}
