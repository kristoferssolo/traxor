use claims::{assert_ok, assert_some};
use ratatui::{Terminal, backend::TestBackend, style::Color};
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
