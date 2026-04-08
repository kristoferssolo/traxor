use claims::assert_ok;
use transmission_rpc::types::{Torrent, TorrentStatus};
use traxor::{app::App, config::Config};

#[test]
fn app_creation() {
    let config = assert_ok!(Config::load());
    let app = assert_ok!(App::new(config));
    assert_eq!(app.tabs().len(), 5);
    assert_eq!(app.tabs()[0].name(), "Overview");
    assert_eq!(app.tabs()[1].name(), "Downloading");
    assert_eq!(app.tabs()[2].name(), "Peers");
    assert_eq!(app.tabs()[3].name(), "History");
    assert_eq!(app.tabs()[4].name(), "Queued");
}

#[test]
fn app_quit() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.quit();
    assert!(!app.running);
}

#[test]
fn app_next_tab() {
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
fn app_prev_tab() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_eq!(app.index(), 0);
    app.prev_tab();
    assert_eq!(app.index(), 4); // Wraps around
    app.prev_tab();
    assert_eq!(app.index(), 3);
}

#[test]
fn app_switch_tab() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_eq!(app.index(), 0);
    app.switch_tab(2);
    assert_eq!(app.index(), 2);
    app.switch_tab(0);
    assert_eq!(app.index(), 0);
}

#[test]
fn app_toggle_popup() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert!(!app.show_help);
    app.toggle_help();
    assert!(app.show_help);
    app.toggle_help();
    assert!(!app.show_help);
}

#[test]
fn app_open_close_popup() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert!(!app.show_help);
    app.open_help();
    assert!(app.show_help);
    app.close_help();
    assert!(!app.show_help);
}

#[test]
fn app_next_uses_filtered_torrents() {
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
fn app_prepare_rename_action_uses_filtered_selection() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta")];
    app.filter_text = "beta".into();
    app.state.select(Some(0));

    app.prepare_rename_action();

    assert_eq!(app.input_handler.text, "beta");
}

#[test]
fn app_filtered_torrents_respects_active_tab_statuses() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![
        torrent_with_status(1, "downloading", TorrentStatus::Downloading),
        torrent_with_status(2, "queued", TorrentStatus::QueuedToDownload),
        torrent_with_status(3, "seeding", TorrentStatus::Seeding),
        torrent_with_status(4, "stopped", TorrentStatus::Stopped),
    ];

    app.switch_tab(1);
    let downloading = app.filtered_torrents();
    assert_eq!(downloading.len(), 2);
    assert_eq!(downloading[0].name.as_deref(), Some("downloading"));
    assert_eq!(downloading[1].name.as_deref(), Some("queued"));

    app.switch_tab(2);
    let seeding = app.filtered_torrents();
    assert_eq!(seeding.len(), 1);
    assert_eq!(seeding[0].name.as_deref(), Some("seeding"));

    app.switch_tab(4);
    let queued = app.filtered_torrents();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].name.as_deref(), Some("queued"));
}

fn torrent(id: i64, name: &str) -> Torrent {
    torrent_with_status(id, name, TorrentStatus::Stopped)
}

fn torrent_with_status(id: i64, name: &str, status: TorrentStatus) -> Torrent {
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
        status: Some(status),
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
