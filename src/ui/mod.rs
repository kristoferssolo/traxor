mod popup;
mod table;

use crate::app::{App, Tab};
use popup::render_popup;
use ratatui::{
    layout::Alignment,
    prelude::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Tabs},
    Frame,
};
use table::render_table;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples

    let size = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let titles: Vec<_> = app
        .tabs()
        .iter()
        .map(|x| Line::from(x.to_string()))
        .collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .select(app.index())
        .style(Style::default().fg(Color::Blue))
        .highlight_style(Style::default().fg(Color::Green))
        .divider("|");

    frame.render_widget(tabs, chunks[0]); // renders tab

    let table = if app.index() == 0 {
        render_table(app, Tab::All)
    } else if app.index() == 1 {
        render_table(app, Tab::Active)
    } else if app.index() == 2 {
        render_table(app, Tab::Downloading)
    } else {
        // Fallback or handle error, though unreachable!() implies this won't happen
        render_table(app, Tab::All) // Default to Tab::All if index is unexpected
    };
    frame.render_stateful_widget(table, chunks[1], &mut app.state); // renders table

    if app.show_popup {
        let block = Block::default().borders(Borders::ALL);
        let size = render_popup(size);
        frame.render_widget(Clear, size);
        frame.render_widget(block, size);
    }
}
