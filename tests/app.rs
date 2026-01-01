use traxor::{app::App, config::Config};

#[test]
fn test_app_creation() {
    let config = Config::load().unwrap();
    let app = App::new(config).unwrap();
    assert_eq!(app.tabs().len(), 5);
}

#[test]
fn test_app_quit() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    app.quit();
    assert!(!app.running);
}

#[test]
fn test_app_next_tab() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert_eq!(app.index(), 0);
    app.next_tab();
    assert_eq!(app.index(), 1);
    app.next_tab();
    assert_eq!(app.index(), 2);
    app.next_tab();
    assert_eq!(app.index(), 3);
    app.next_tab();
    assert_eq!(app.index(), 4);
    app.next_tab();
    assert_eq!(app.index(), 0); // Wraps around
}

#[test]
fn test_app_prev_tab() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert_eq!(app.index(), 0);
    app.prev_tab();
    assert_eq!(app.index(), 4); // Wraps around
    app.prev_tab();
    assert_eq!(app.index(), 3);
}

#[test]
fn test_app_switch_tab() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert_eq!(app.index(), 0);
    app.switch_tab(2);
    assert_eq!(app.index(), 2);
    app.switch_tab(0);
    assert_eq!(app.index(), 0);
}

#[test]
fn test_app_toggle_popup() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert!(!app.show_help);
    app.toggle_help();
    assert!(app.show_help);
    app.toggle_help();
    assert!(!app.show_help);
}

#[test]
fn test_app_open_close_popup() {
    let config = Config::load().unwrap();
    let mut app = App::new(config).unwrap();
    assert!(!app.show_help);
    app.open_help();
    assert!(app.show_help);
    app.close_help();
    assert!(!app.show_help);
}
