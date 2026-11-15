use claims::{assert_err, assert_none, assert_ok, assert_some_eq};
use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};
use transmission_rpc::types::{Torrent, TorrentGetField, TorrentStatus};
use traxor::{
    app::{App, InputMode, types::DeleteTarget},
    config::Config,
};

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
    assert!(
        app.tabs()[0]
            .fields()
            .contains(&TorrentGetField::DownloadDir)
    );
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
    app.next();

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

#[test]
fn app_prepare_delete_keeps_captured_targets_across_refresh() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta"), torrent(3, "gamma")];
    app.next();
    app.next();

    app.prepare_delete(true);
    app.torrents.torrents = vec![
        torrent(4, "delta"),
        torrent(1, "alpha"),
        torrent(3, "gamma"),
    ];

    let target = delete_target(&app);
    assert_eq!(target.ids().collect::<Vec<_>>(), [2]);
    assert_eq!(target.names().collect::<Vec<_>>(), ["beta"]);
    assert!(target.delete_local_data());
}

#[test]
fn app_prepare_delete_captures_visible_selection_in_row_order() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta"), torrent(3, "gamma")];
    app.torrents.selected = [3, 1].into();
    app.next();

    app.prepare_delete(false);

    let target = delete_target(&app);
    assert_eq!(target.ids().collect::<Vec<_>>(), [1, 3]);
    assert_eq!(target.names().collect::<Vec<_>>(), ["alpha", "gamma"]);
    assert!(!target.delete_local_data());
}

#[test]
fn app_prepare_delete_does_not_open_without_visible_targets() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta")];

    app.prepare_delete(false);
    assert_eq!(app.input_mode, InputMode::None);

    app.torrents.selected = [2].into();
    app.filter_text = "alpha".into();
    app.next();
    app.prepare_delete(false);
    assert_eq!(app.input_mode, InputMode::None);
}

#[test]
fn app_highlight_follows_torrent_across_reorder() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta"), torrent(3, "gamma")];
    app.next();
    app.next();

    app.torrents.torrents = vec![torrent(2, "beta"), torrent(3, "gamma"), torrent(1, "alpha")];
    app.prepare_rename_action();
    assert_eq!(app.input_handler.text, "beta");

    app.sync_highlight();
    assert_some_eq!(app.state.selected(), 0);
}

#[test]
fn app_highlight_clamps_when_torrent_disappears() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta"), torrent(3, "gamma")];
    app.next();
    app.next();
    app.next();

    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta")];
    app.prepare_rename_action();
    assert_eq!(app.input_handler.text, "");

    app.input_handler.clear();
    app.sync_highlight();
    assert_some_eq!(app.state.selected(), 1);
    app.prepare_rename_action();
    assert_eq!(app.input_handler.text, "beta");
}

#[test]
fn app_highlight_clears_when_visible_rows_become_empty() {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = vec![torrent_with_status(1, "alpha", TorrentStatus::Downloading)];
    app.switch_tab(1);
    app.next();

    app.torrents.torrents = vec![torrent(1, "alpha")];
    app.sync_highlight();
    assert_none!(app.state.selected());

    app.torrents.torrents = vec![torrent_with_status(1, "alpha", TorrentStatus::Downloading)];
    app.prepare_delete(false);
    assert_eq!(app.input_mode, InputMode::None);
    app.prepare_rename_action();
    assert_eq!(app.input_handler.text, "");
}

#[tokio::test]
async fn app_confirm_delete_sends_captured_targets() {
    let (url, request_body) = capture_rpc_request();
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_ok!(app.torrents.url(&url));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta"), torrent(3, "gamma")];
    app.next();
    app.next();

    app.prepare_delete(true);
    app.torrents.torrents = vec![
        torrent(4, "delta"),
        torrent(1, "alpha"),
        torrent(3, "gamma"),
    ];
    assert_ok!(app.confirm_delete().await);

    let body = assert_ok!(request_body.recv_timeout(Duration::from_secs(5)));
    assert_eq!(
        body,
        r#"{"method":"torrent-remove","arguments":{"ids":[2],"delete-local-data":true}}"#
    );
    assert_eq!(app.input_mode, InputMode::None);
}

#[tokio::test]
async fn app_rename_does_not_retarget_when_its_torrent_disappears() {
    let (url, request_body) = capture_rpc_request();
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_ok!(app.torrents.url(&url));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta")];
    app.next();
    app.next();
    app.prepare_rename_action();
    app.input_handler.set_text("renamed".to_owned());

    // A refresh drops the torrent being renamed; the highlight falls back to "alpha".
    app.torrents.torrents = vec![torrent(1, "alpha")];
    app.sync_highlight();
    assert_ok!(app.rename_torrent().await);

    // A request would have been answered, and its body sent, before `rename_torrent` returned.
    assert_err!(request_body.try_recv());
    assert_eq!(app.input_mode, InputMode::None);
}

#[tokio::test]
async fn app_move_sends_targets_captured_when_prompt_opened() {
    let (url, request_body) = capture_rpc_request();
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    assert_ok!(app.torrents.url(&url));
    app.torrents.torrents = vec![torrent(1, "alpha"), torrent(2, "beta")];
    app.next();
    app.next();
    app.prepare_move_action();
    app.input_handler.set_text("/data".to_owned());

    app.torrents.torrents = vec![torrent(1, "alpha")];
    app.sync_highlight();
    assert_ok!(app.move_torrent().await);

    let body = assert_ok!(request_body.recv_timeout(Duration::from_secs(5)));
    assert!(body.contains(r#""ids":[2]"#), "{body}");
}

/// Answers one Transmission RPC call on an ephemeral local port.
///
/// Returns the RPC URL and a receiver for the request body. The listener stays bound until the
/// call is served, and reads time out so a stalled client cannot hang the test.
fn capture_rpc_request() -> (String, Receiver<String>) {
    let listener = assert_ok!(TcpListener::bind("127.0.0.1:0"));
    let url = format!(
        "http://{}/transmission/rpc",
        assert_ok!(listener.local_addr())
    );
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let (stream, _) = assert_ok!(listener.accept());
        assert_ok!(stream.set_read_timeout(Some(Duration::from_secs(5))));
        let mut reader = BufReader::new(&stream);
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            assert_ok!(reader.read_line(&mut line));
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some((name, value)) = line.split_once(':')
                && name.eq_ignore_ascii_case("content-length")
            {
                content_length = assert_ok!(value.trim().parse());
            }
        }
        let mut body = vec![0; content_length];
        assert_ok!(reader.read_exact(&mut body));
        assert_ok!(sender.send(assert_ok!(String::from_utf8(body))));

        let response = r#"{"arguments":{},"result":"success"}"#;
        let mut writer = &stream;
        assert_ok!(write!(
            writer,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
            response.len()
        ));
    });
    (url, receiver)
}

fn delete_target(app: &App) -> &DeleteTarget {
    match &app.input_mode {
        InputMode::ConfirmDelete(target) => target,
        other => panic!("expected delete confirmation, got {other:?}"),
    }
}

#[test]
fn app_first_and_last_jump_without_wrapping() {
    let mut app = app_with_torrents(5);
    app.state.select(Some(2));
    app.open_help();

    app.last();
    assert_some_eq!(app.state.selected(), 4);
    assert!(!app.show_help);
    app.last();
    assert_some_eq!(app.state.selected(), 4);

    app.first();
    assert_some_eq!(app.state.selected(), 0);
    app.first();
    assert_some_eq!(app.state.selected(), 0);
}

#[test]
fn app_jump_and_page_survive_highlight_sync() {
    let mut app = app_with_torrents(5);
    app.next();

    // Each draw re-syncs the table row from the highlighted torrent's id.
    app.last();
    app.sync_highlight();
    assert_some_eq!(app.state.selected(), 4);

    app.page_up();
    app.sync_highlight();
    assert_some_eq!(app.state.selected(), 0);

    app.page_down();
    app.sync_highlight();
    assert_some_eq!(app.state.selected(), 4);

    app.first();
    app.sync_highlight();
    assert_some_eq!(app.state.selected(), 0);
}

#[test]
fn app_last_uses_filtered_torrents() {
    let mut app = app_with_torrents(5);
    app.filter_text = "torrent-3".into();

    app.last();

    assert_some_eq!(app.state.selected(), 0);
}

#[test]
fn app_page_moves_by_page_size_and_stops_at_ends() {
    let mut app = app_with_torrents(25);
    app.set_page_size(10);

    app.page_down();
    assert_some_eq!(app.state.selected(), 10);
    app.page_down();
    assert_some_eq!(app.state.selected(), 20);
    app.page_down();
    assert_some_eq!(app.state.selected(), 24);

    app.page_up();
    assert_some_eq!(app.state.selected(), 14);
    app.page_up();
    assert_some_eq!(app.state.selected(), 4);
    app.page_up();
    assert_some_eq!(app.state.selected(), 0);
}

#[test]
fn app_page_larger_than_list_clamps_to_ends() {
    let mut app = app_with_torrents(3);
    app.state.select(Some(1));

    app.page_down();
    assert_some_eq!(app.state.selected(), 2);
    app.page_up();
    assert_some_eq!(app.state.selected(), 0);
}

#[test]
fn app_page_clamps_stale_highlight() {
    let mut app = app_with_torrents(3);
    app.set_page_size(1);
    app.state.select(Some(10));

    app.page_up();

    assert_some_eq!(app.state.selected(), 1);
}

#[test]
fn app_page_size_is_at_least_one_row() {
    let mut app = app_with_torrents(3);
    app.set_page_size(0);
    app.state.select(Some(0));

    app.page_down();

    assert_some_eq!(app.state.selected(), 1);
}

#[test]
fn app_jumps_are_noops_on_empty_list() {
    let mut app = app_with_torrents(0);
    app.open_help();

    app.first();
    app.last();
    app.page_down();
    app.page_up();

    assert_none!(app.state.selected());
    assert!(app.show_help);
}

#[test]
fn app_select_all_selects_only_visible_torrents() {
    let mut app = app_with_torrents(0);
    app.torrents.torrents = vec![
        torrent_with_status(1, "alpha", TorrentStatus::Downloading),
        torrent_with_status(2, "beta", TorrentStatus::Downloading),
        torrent_with_status(3, "gamma", TorrentStatus::Seeding),
    ];
    app.switch_tab(1);
    app.filter_text = "beta".into();
    app.open_help();

    app.select_all();

    assert_eq!(app.torrents.selected, HashSet::from([2]));
    assert!(!app.show_help);
    assert_none!(app.state.selected());
}

#[test]
fn app_select_all_keeps_existing_selection() {
    let mut app = app_with_torrents(3);
    app.torrents.selected.insert(99);

    app.select_all();

    assert_eq!(app.torrents.selected, HashSet::from([0, 1, 2, 99]));
}

#[test]
fn app_select_all_is_noop_on_empty_list() {
    let mut app = app_with_torrents(0);
    app.open_help();

    app.select_all();

    assert!(app.torrents.selected.is_empty());
    assert!(app.show_help);
}

#[test]
fn app_clear_selection_clears_hidden_torrents() {
    let mut app = app_with_torrents(3);
    app.torrents.selected.extend([0, 1, 2]);
    app.filter_text = "torrent-1".into();
    app.open_help();

    app.clear_selection();

    assert!(app.torrents.selected.is_empty());
    assert!(!app.show_help);
}

fn app_with_torrents(count: i64) -> App {
    let config = assert_ok!(Config::load());
    let mut app = assert_ok!(App::new(config));
    app.torrents.torrents = (0..count)
        .map(|id| torrent(id, &format!("torrent-{id}")))
        .collect();
    app
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
