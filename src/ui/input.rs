use crate::app::{App, InputMode};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};
use tracing::warn;

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();
    let input_area = Rect::new(size.width / 4, size.height / 2 - 1, size.width / 2, 3);

    let title = match app.input_mode {
        InputMode::Move => "Move to",
        InputMode::Rename => "Rename",
        InputMode::None => return,
    };

    let block = Block::default().title(title).borders(Borders::ALL);
    f.render_widget(Clear, input_area);
    f.render_widget(block, input_area);

    let input = Paragraph::new(app.input_handler.text.as_str()).block(Block::default());
    f.render_widget(
        input,
        input_area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
    );

    let cursor_offset = u16::try_from(app.input_handler.cursor_position).unwrap_or_else(|_| {
        warn!("cursor_position out of range, clamping");
        0
    });

    f.set_cursor_position(Position::new(
        input_area.x + cursor_offset + 1,
        input_area.y + 1,
    ));
}
