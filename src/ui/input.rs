use crate::app::{App, InputMode};
use ratatui::{
    prelude::*,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};
use tracing::warn;

pub fn render(f: &mut Frame, app: &App) {
    match app.input_mode {
        InputMode::Move | InputMode::Rename => render_text_input(f, app),
        InputMode::Filter => render_filter_input(f, app),
        InputMode::ConfirmDelete(delete_local_data) => render_confirm_delete(f, delete_local_data),
        InputMode::None => {}
    }
}

fn render_text_input(f: &mut Frame, app: &App) {
    let size = f.area();
    let input_area = Rect::new(size.width / 4, size.height / 2 - 1, size.width / 2, 3);

    let title = match app.input_mode {
        InputMode::Move => "Move to",
        InputMode::Rename => "Rename",
        _ => return,
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
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

fn render_filter_input(f: &mut Frame, app: &App) {
    let size = f.area();
    let width = size.width.min(60);
    let input_area = Rect::new((size.width.saturating_sub(width)) / 2, 1, width, 3);

    let filtered = app.filtered_torrents().len();
    let total = app.torrents.len();

    let block = Block::default()
        .title(" Search ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .title_alignment(Alignment::Left)
        .title_bottom(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                format!("{filtered}/{total}"),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" matches "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(Clear, input_area);
    f.render_widget(block, input_area);

    let prompt = Span::styled("> ", Style::default().fg(Color::Cyan).bold());
    let text = Span::raw(app.input_handler.text.as_str());
    let input = Paragraph::new(Line::from(vec![prompt, text]));

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

    // +2 for the "> " prompt
    f.set_cursor_position(Position::new(
        input_area.x + cursor_offset + 3,
        input_area.y + 1,
    ));
}

fn render_confirm_delete(f: &mut Frame, delete_local_data: bool) {
    let size = f.area();
    let dialog_width = 40;
    let dialog_height = 5;
    let dialog_area = Rect::new(
        (size.width.saturating_sub(dialog_width)) / 2,
        (size.height.saturating_sub(dialog_height)) / 2,
        dialog_width.min(size.width),
        dialog_height.min(size.height),
    );

    let title = if delete_local_data {
        "Delete with data?"
    } else {
        "Delete torrent?"
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::Red).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    f.render_widget(Clear, dialog_area);
    f.render_widget(block, dialog_area);

    let first_line = if delete_local_data {
        "This will delete local files!"
    } else {
        "Remove from list?"
    };

    let text = Paragraph::new(vec![Line::from(first_line), Line::from("(y)es / (n)o")])
        .alignment(Alignment::Center);

    f.render_widget(
        text,
        dialog_area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
    );
}
