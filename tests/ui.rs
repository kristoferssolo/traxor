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
