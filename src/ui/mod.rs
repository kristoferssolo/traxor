mod help;
mod input;
mod table;

use crate::{
    app::{App, Tab},
    config::color::ColorConfig,
};
use help::render_help;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Tabs},
};
use std::str::FromStr;
use table::build_table;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples

    let size = frame.area();
    let tab_style = tab_style(&app.config.colors);
    let highlighted_tab_style = highlighted_tab_style(&app.config.colors);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let titles = app
        .tabs()
        .iter()
        .map(|x| Line::from(x.to_string()))
        .collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .block(default_block())
        .select(app.index())
        .style(tab_style)
        .highlight_style(highlighted_tab_style)
        .divider("|");

    frame.render_widget(tabs, chunks[0]); // renders tab

    app.torrents.set_fields(None);
    let torrents = &app.torrents.torrents;
    let selected = &app.torrents.selected;
    let colors = &app.config.colors;

    let table = build_table(torrents, selected, colors, Tab::from(app.index()).fields());

    frame.render_stateful_widget(table, chunks[1], &mut app.state);

    if app.show_help {
        render_help(frame, app);
    }

    if app.input_mode {
        input::render(frame, app);
    }
}

#[must_use]
pub fn to_color(value: &str) -> Color {
    Color::from_str(value).unwrap_or_default()
}

fn tab_style(cfg: &ColorConfig) -> Style {
    let fg = to_color(&cfg.info_foreground);
    Style::default().fg(fg)
}

fn highlighted_tab_style(cfg: &ColorConfig) -> Style {
    let fg = to_color(&cfg.header_foreground);
    Style::default().fg(fg)
}

fn default_block() -> Block<'static> {
    Block::default()
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
}
