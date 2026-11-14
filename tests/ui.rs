use claims::{assert_ok, assert_some};
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    style::{Color, Modifier},
};
use toml::Table;
use transmission_rpc::types::{Torrent, TorrentGetField};
use traxor::{app::App, config::Config, ui};

#[test]
fn status_bar_shows_error_in_red() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.show_error("rename failed:\ninvalid name");
    let mut terminal = assert_ok!(Terminal::new(TestBackend::new(80, 10)));

    assert_ok!(terminal.draw(|frame| ui::render(&mut app, frame)));

    let buffer = terminal.backend().buffer();
    let status_line = (0..buffer.area.width)
        .map(|x| buffer[(x, 8)].symbol())
        .collect::<String>();
    let column = assert_some!(status_line.find("rename failed: invalid name"));
    let column = u16::try_from(column).expect("column fits in u16");
    assert_eq!(buffer[(column, 8)].fg, Color::Red);
}

#[test]
fn status_bar_truncates_long_error_and_keeps_stats() {
    let status_line = render_status_line_with_error(&"rename failed ".repeat(20));

    assert!(status_line.contains("rename failed"), "{status_line:?}");
    assert!(status_line.contains('…'), "{status_line:?}");
    assert_stats_visible(&status_line);
}

#[test]
fn status_bar_truncates_wide_unicode_error_and_keeps_stats() {
    let status_line = render_status_line_with_error(&"重命名失败".repeat(20));

    // Wide characters fill a second, blank cell in the buffer.
    assert!(status_line.contains("重 命 名 失 败"), "{status_line:?}");
    assert!(status_line.contains('…'), "{status_line:?}");
    assert_stats_visible(&status_line);
}

#[test]
fn status_bar_truncates_ligature_error_and_keeps_stats() {
    // Each lam-alef pair measures one cell as a string but renders into two.
    let status_line = render_status_line_with_error(&"لا".repeat(40));

    assert!(status_line.contains('…'), "{status_line:?}");
    assert_stats_visible(&status_line);
}

#[test]
fn status_bar_truncates_halfwidth_kana_error_and_keeps_stats() {
    // Halfwidth sound marks take a cell of their own when rendered.
    let status_line = render_status_line_with_error(&"ｶﾞ".repeat(40));

    assert!(status_line.contains('…'), "{status_line:?}");
    assert_stats_visible(&status_line);
}

const WIDTH: u16 = 80;
const STATUS_ROW: u16 = 8;

/// Renders the UI with `error` in the status bar and returns the status text row.
fn render_status_line_with_error(error: &str) -> String {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.show_error(error);
    let mut terminal = assert_ok!(Terminal::new(TestBackend::new(WIDTH, 10)));

    assert_ok!(terminal.draw(|frame| ui::render(&mut app, frame)));

    let buffer = terminal.backend().buffer();
    let ellipsis_column = (0..WIDTH).find(|&x| buffer[(x, STATUS_ROW)].symbol() == "…");
    let ellipsis_column = assert_some!(ellipsis_column);
    assert_eq!(buffer[(ellipsis_column, STATUS_ROW)].fg, Color::Red);
    (0..WIDTH)
        .map(|x| buffer[(x, STATUS_ROW)].symbol())
        .collect()
}

/// Asserts the stats end the line right before the closing border.
fn assert_stats_visible(status_line: &str) {
    let inner = status_line.trim_end_matches('│').trim_end();
    assert!(inner.contains(" │ ↓"), "{status_line:?}");
    let uploaded = assert_some!(inner.rsplit_once("U:")).1;
    assert!(uploaded.ends_with('B'), "{status_line:?}");
}

const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");

const ACTION_LABELS: [&str; 26] = [
    "Up/down",
    "First/last",
    "Page",
    "Prev/next tab",
    "Switch tab",
    "Filter",
    "Clear filter",
    "Close help",
    "Quit",
    "Start/stop",
    "Start/stop all",
    "Multi-select",
    "Select visible",
    "Clear selection",
    "Move",
    "Rename",
    "Remove",
    "Delete with data",
    "Move cursor",
    "Start/end",
    "Delete forward",
    "Delete word",
    "Delete to start",
    "Complete path",
    "Submit",
    "Cancel",
];

/// App built from the embedded defaults with `keybinds` overrides, ignoring user config.
fn app_with_keybinds(keybinds: &[(&str, &str)]) -> App {
    let mut config: Table = assert_ok!(toml::from_str(DEFAULT_CONFIG));
    let table = assert_some!(config.get_mut("keybinds").and_then(|v| v.as_table_mut()));
    for (name, value) in keybinds {
        table.insert((*name).to_owned(), (*value).into());
    }
    let config: Config = assert_ok!(config.try_into());
    assert_ok!(App::new(config))
}

fn help_app() -> App {
    let mut app = app_with_keybinds(&[]);
    app.open_help();
    app
}

/// Renders `app` at `width`x`height` and returns the screen as text rows.
fn render(app: &mut App, width: u16, height: u16) -> Vec<String> {
    let mut terminal = assert_ok!(Terminal::new(TestBackend::new(width, height)));
    assert_ok!(terminal.draw(|frame| ui::render(app, frame)));
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect())
        .collect()
}

/// Row and column where `text` first appears on `screen`.
fn find(screen: &[String], text: &str) -> (usize, usize) {
    let found = screen.iter().enumerate().find_map(|(row, line)| {
        line.find(text)
            .map(|byte| (row, line[..byte].chars().count()))
    });
    assert_some!(found, "{text:?} not on screen:\n{}", screen.join("\n"))
}

fn contains(screen: &[String], text: &str) -> bool {
    screen.iter().any(|line| line.contains(text))
}

#[test]
fn help_page_shows_every_action_at_80x24() {
    let screen = render(&mut help_app(), 80, 24);

    for label in ACTION_LABELS {
        find(&screen, label);
    }
    let (nav_row, nav_col) = find(&screen, "Navigation");
    let (actions_row, actions_col) = find(&screen, "Actions");
    let (prompts_row, prompts_col) = find(&screen, "In prompts");
    assert_eq!(nav_row, actions_row);
    assert_eq!(nav_row, prompts_row);
    assert!(nav_col < actions_col && actions_col < prompts_col);
}

#[test]
fn help_page_replaces_table() {
    let mut app = app_with_keybinds(&[]);
    assert!(contains(&render(&mut app, 80, 24), "Status"));

    app.open_help();
    let screen = render(&mut app, 80, 24);

    assert!(!contains(&screen, "Status"));
    find(&screen, "Overview");
}

#[test]
fn status_bar_shows_help_mode() {
    let screen = render(&mut help_app(), 80, 24);
    let status = &screen[21..];

    find(status, "? Close help");
    find(status, " HELP ");
}

#[test]
fn help_page_shows_configured_keys() {
    let mut app = app_with_keybinds(&[
        ("prev_torrent", "up"),
        ("next_torrent", "down"),
        ("move_torrent", "ctrl+o"),
        ("toggle_help", "F1"),
    ]);
    app.open_help();
    let screen = render(&mut app, 80, 24);

    find(&screen, "up/down");
    find(&screen, "ctrl+o Move");
    find(&screen, "F1 Close help");
    assert!(!contains(&screen, "k/j"));
    assert!(!contains(&screen, "m      Move"));
}

#[test]
fn help_page_stacks_into_two_columns() {
    let screen = render(&mut help_app(), 64, 32);

    for label in ACTION_LABELS {
        find(&screen, label);
    }
    let (nav_row, nav_col) = find(&screen, "Navigation");
    let (actions_row, actions_col) = find(&screen, "Actions");
    let (prompts_row, prompts_col) = find(&screen, "In prompts");
    assert_eq!(nav_row, actions_row);
    assert!(nav_col < actions_col);
    assert_eq!(actions_col, prompts_col);
    assert!(actions_row < prompts_row);
}

#[test]
fn help_page_stacks_into_one_column_when_narrow() {
    let screen = render(&mut help_app(), 40, 20);

    let (nav_row, nav_col) = find(&screen, "Navigation");
    let (search_row, search_col) = find(&screen, "Search");
    assert_eq!(nav_col, search_col);
    assert!(nav_row < search_row);
    assert!(!screen[nav_row].contains("Actions"));
}

#[test]
fn help_page_survives_tiny_terminal() {
    render(&mut help_app(), 10, 5);
    render(&mut help_app(), 1, 1);
}

/// App on the embedded default config, independent of any user config.
fn app() -> App {
    let config: Config = assert_ok!(toml::from_str(include_str!("../config/default.toml")));
    assert_ok!(App::new(config))
}

fn torrent(id: i64, fields: &str) -> Torrent {
    assert_ok!(toml::from_str(&format!("id = {id}\n{fields}")))
}

fn draw(app: &mut App, width: u16) -> Buffer {
    let mut terminal = assert_ok!(Terminal::new(TestBackend::new(width, 20)));
    assert_ok!(terminal.draw(|frame| ui::render(app, frame)));
    terminal.backend().buffer().clone()
}

fn lines(buffer: &Buffer) -> Vec<String> {
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect()
        })
        .collect()
}

/// Cell position of the first occurrence of `text`.
fn find_in_buffer(buffer: &Buffer, text: &str) -> Option<(u16, u16)> {
    lines(buffer).iter().zip(0..).find_map(|(line, y)| {
        let byte = line.find(text)?;
        let x = u16::try_from(line[..byte].chars().count()).ok()?;
        Some((x, y))
    })
}

#[test]
fn sorted_column_header_shows_direction_and_highlight() {
    let mut app = app();
    app.torrents.torrents = vec![
        torrent(
            1,
            r#"name = "small"
totalSize = 10"#,
        ),
        torrent(
            2,
            r#"name = "large"
totalSize = 1000"#,
        ),
    ];
    let size_column = app.tabs()[0]
        .fields()
        .iter()
        .position(|&field| field == TorrentGetField::TotalSize)
        .expect("overview shows the size column");
    for _ in 0..=size_column {
        app.cycle_sort();
    }

    let buffer = draw(&mut app, 200);
    let (x, y) =
        find_in_buffer(&buffer, "Total Size ▼").expect("sorted header has a descending arrow");
    let sorted = &buffer[(x, y)];
    assert_eq!(sorted.fg, Color::Black);
    assert_eq!(sorted.bg, Color::Yellow);
    assert!(sorted.modifier.contains(Modifier::BOLD));
    assert!(!sorted.modifier.contains(Modifier::UNDERLINED));

    let (x, y) = find_in_buffer(&buffer, "Status").expect("status header");
    assert_eq!(buffer[(x, y)].fg, Color::Yellow);
    assert_ne!(buffer[(x, y)].bg, Color::Yellow);
    assert!(find_in_buffer(&buffer, "▲").is_none());

    let large = find_in_buffer(&buffer, "large").expect("large row");
    let small = find_in_buffer(&buffer, "small").expect("small row");
    assert!(large.1 < small.1);

    app.reverse_sort();
    let buffer = draw(&mut app, 200);
    assert!(find_in_buffer(&buffer, "Total Size ▲").is_some());
}
