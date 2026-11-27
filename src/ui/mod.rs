mod help;
pub mod hit_test;
mod input;
mod status;
mod table;

use crate::{
    app::{App, InputMode, RenderedAreas},
    config::color::ColorConfig,
};
use help::render_help;
use ratatui::{
    prelude::*,
    style::Modifier,
    widgets::{Block, BorderType, Borders, Tabs},
};
use std::str::FromStr;
use table::build_table;

/// Padding on each side of a tab title.
const TAB_PADDING: &str = " ";
/// Separator between tab titles.
const TAB_DIVIDER: &str = "|";

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let size = frame.area();
    let tab_style = tab_style(&app.config.colors);
    let highlighted_tab_style = highlighted_tab_style(&app.config.colors);

    let [tabs_area, body_area, status_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(size);

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
        .padding(TAB_PADDING, TAB_PADDING)
        .divider(TAB_DIVIDER);

    frame.render_widget(tabs, tabs_area);

    app.sync_highlight();
    app.set_page_size(hit_test::table_body_rows(body_area));

    if app.show_help {
        render_help(frame, app, body_area);
    } else if let Some(tab) = app.tabs().get(app.index()) {
        let torrents = app.filtered_torrents();
        let table_rows = torrents.iter().map(|torrent| torrent.id).collect();
        let selected = &app.torrents.selected;

        let table = build_table(
            &torrents,
            selected,
            app.state.selected(),
            &app.config,
            tab.fields(),
            body_area,
            tab.sort(),
        );
        frame.render_stateful_widget(table, body_area, &mut app.state);
        app.areas = RenderedAreas {
            tabs: tabs_area,
            table: body_area,
            table_offset: app.state.offset(),
            table_rows,
        };
    }

    status::render(frame, app, status_area);

    if app.input_mode != InputMode::None {
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
    Style::default()
        .fg(fg)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

fn default_block() -> Block<'static> {
    Block::default()
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
}
