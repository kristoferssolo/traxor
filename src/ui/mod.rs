mod help;
mod table;

use crate::app::{App, Tab};
use help::render_help;
use ratatui::{prelude::*, widgets::*};
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
        .style(
            Style::default().fg(app
                .config
                .colors
                .get_color(&app.config.colors.info_foreground)),
        )
        .highlight_style(
            Style::default().fg(app
                .config
                .colors
                .get_color(&app.config.colors.warning_foreground)),
        )
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

    if app.show_help {
        render_help(frame, app);
    }
}
