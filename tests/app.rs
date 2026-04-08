use claims::assert_ok;
use transmission_rpc::types::Torrent;
use traxor::{app::App, config::Config};

#[test]
fn test_app_creation() {
    let config = assert_ok!(Config::load());
    let app = assert_ok!(App::new(config));
    assert_eq!(app.tabs().len(), 5);
    assert_eq!(app.tabs()[0].name(), "Overview");
    assert_eq!(app.tabs()[1].name(), "Transfer");
    assert_eq!(app.tabs()[2].name(), "Peers");
    assert_eq!(app.tabs()[3].name(), "History");
    assert_eq!(app.tabs()[4].name(), "Storage");
}

#[test]
fn test_app_quit() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.quit();
    assert!(!app.running);
}

#[test]
fn test_app_next_tab() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
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
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_eq!(app.index(), 0);
    app.prev_tab();
    assert_eq!(app.index(), 4); // Wraps around
    app.prev_tab();
    assert_eq!(app.index(), 3);
}

#[test]
fn test_app_switch_tab() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_eq!(app.index(), 0);
    app.switch_tab(2);
    assert_eq!(app.index(), 2);
    app.switch_tab(0);
    assert_eq!(app.index(), 0);
}

#[test]
fn test_app_toggle_popup() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert!(!app.show_help);
    app.toggle_help();
    assert!(app.show_help);
    app.toggle_help();
    assert!(!app.show_help);
}

#[test]
fn test_app_open_close_popup() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert!(!app.show_help);
    app.open_help();
    assert!(app.show_help);
    app.close_help();
    assert!(!app.show_help);
}

#[test]
fn test_app_next_uses_filtered_torrents() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta")];
    app.filter_text = "beta".into();
    app.state.select(Some(0));

    app.next();

    assert_eq!(app.index(), 0);
    assert_eq!(app.state.selected(), Some(0));
}

#[test]
fn test_app_prepare_rename_action_uses_filtered_selection() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta")];
    app.filter_text = "beta".into();
    app.state.select(Some(0));

    app.prepare_rename_action();

    assert_eq!(app.input_handler.text, "beta");
}

fn torrent(id: i64, name: &str) -> Torrent {
    Torrent {
        activity_date: None,
        added_date: None,
        availability: None,
        bandwidth_priority: None,
        comment: None,
        corrupt_ever: None,
        creator: None,
        date_created: None,
        desired_available: None,
        done_date: None,
        download_dir: None,
        downloaded_ever: None,
        download_limit: None,
        download_limited: None,
        edit_date: None,
        error: None,
        error_string: None,
        eta: None,
        eta_idle: None,
        group: None,
        hash_string: None,
        have_unchecked: None,
        have_valid: None,
        honors_session_limits: None,
        id: Some(id),
        is_finished: None,
        is_private: None,
        is_stalled: None,
        labels: None,
        left_until_done: None,
        magnet_link: None,
        manual_announce_time: None,
        max_connected_peers: None,
        metadata_percent_complete: None,
        name: Some(name.into()),
        peer_limit: None,
        peers: None,
        peers_connected: None,
        peers_from: None,
        peers_getting_from_us: None,
        peers_sending_to_us: None,
        percent_complete: None,
        percent_done: None,
        pieces: None,
        piece_count: None,
        piece_size: None,
        primary_mime_type: None,
        queue_position: None,
        rate_download: None,
        rate_upload: None,
        recheck_progress: None,
        seconds_downloading: None,
        seconds_seeding: None,
        seed_idle_limit: None,
        seed_idle_mode: None,
        seed_ratio_limit: None,
        seed_ratio_mode: None,
        sequential_download: None,
        size_when_done: None,
        start_date: None,
        status: None,
        torrent_file: None,
        total_size: None,
        trackers: None,
        tracker_list: None,
        tracker_stats: None,
        upload_ratio: None,
        uploaded_ever: None,
        upload_limit: None,
        upload_limited: None,
        files: None,
        wanted: None,
        webseeds: None,
        webseeds_sending_to_us: None,
        priorities: None,
        file_stats: None,
        file_count: None,
    }
}
