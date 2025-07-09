use crate::app::App;
use ratatui::{prelude::*, widgets::*};

pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let input_area = Rect::new(size.width / 4, size.height / 2 - 1, size.width / 2, 3);

    let block = Block::default().title("Move to").borders(Borders::ALL);
    f.render_widget(Clear, input_area);
    f.render_widget(block, input_area);

    let input = Paragraph::new(app.input.as_str()).block(Block::default());
    f.render_widget(
        input,
        input_area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
    );

    f.set_cursor_position(ratatui::layout::Position::new(
        input_area.x + app.cursor_position as u16 + 1,
        input_area.y + 1,
    ));
}
