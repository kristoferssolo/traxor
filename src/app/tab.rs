use crate::{app::sort::Sort, config::tabs::TabConfig};
use std::fmt::Display;
use transmission_rpc::types::{Torrent, TorrentGetField, TorrentStatus};

/// Fields the app reads regardless of the configured columns: `id` for selection and
/// RPC actions, `name` for filtering and renaming, `status` for tab matching, row
/// colors, and toggling, `downloadDir` for the move prefill, `percentDone` for the
/// progress underline across the Name column, and the rate and transfer totals for the
/// status bar.
const APP_FIELDS: [TorrentGetField; 9] = [
    TorrentGetField::Id,
    TorrentGetField::Name,
    TorrentGetField::Status,
    TorrentGetField::DownloadDir,
    TorrentGetField::PercentDone,
    TorrentGetField::RateDownload,
    TorrentGetField::RateUpload,
    TorrentGetField::DownloadedEver,
    TorrentGetField::UploadedEver,
];

/// A tab with name and column configuration.
#[derive(Debug, Clone)]
pub struct Tab {
    config: TabConfig,
    fields: Vec<TorrentGetField>,
    statuses: Vec<TorrentStatus>,
    sort: Option<Sort>,
}

impl Tab {
    /// Create a new tab from a config that passed [`TabConfig::validate`].
    ///
    /// Unknown statuses would leave the tab matching every torrent, so only
    /// [`crate::app::App::new`], which validates first, builds tabs.
    #[must_use]
    pub(crate) fn new(config: TabConfig) -> Self {
        let fields = config.fields();
        let statuses = config.statuses();
        Self {
            config,
            fields,
            statuses,
            sort: None,
        }
    }

    /// Returns the column fields for this tab.
    #[must_use]
    pub fn fields(&self) -> &[TorrentGetField] {
        &self.fields
    }

    /// Returns the tab name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Returns the active column sort, or [`None`] for daemon order.
    #[must_use]
    pub const fn sort(&self) -> Option<Sort> {
        self.sort
    }

    /// Sorts by the next column in its natural direction.
    ///
    /// Cycles from daemon order through each column in order, then back to daemon order.
    pub fn cycle_sort(&mut self) {
        let column = self.sort.map_or(0, |sort| sort.column.saturating_add(1));
        self.sort = self
            .fields
            .get(column)
            .map(|&field| Sort::new(column, field));
    }

    /// Flips the direction of the active sort. Does nothing in daemon order.
    pub fn reverse_sort(&mut self) {
        self.sort = self.sort.map(Sort::reversed);
    }

    /// Returns `true` if the torrent belongs in this tab.
    #[must_use]
    pub fn matches(&self, torrent: &Torrent) -> bool {
        self.statuses.is_empty()
            || torrent
                .status
                .is_some_and(|status| self.statuses.contains(&status))
    }
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.config.name)
    }
}

/// Returns the deduplicated torrent fields to request from Transmission: the app's own
/// fields plus every column of every tab, so switching tabs never shows empty columns.
#[must_use]
pub fn required_fields(tabs: &[Tab]) -> Vec<TorrentGetField> {
    let columns = tabs.iter().flat_map(|tab| tab.fields().iter().copied());
    APP_FIELDS
        .into_iter()
        .chain(columns)
        .fold(Vec::new(), |mut fields, field| {
            if !fields.contains(&field) {
                fields.push(field);
            }
            fields
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::{App, sort::SortDirection},
        config::Config,
    };
    use claims::{assert_err, assert_none, assert_ok, assert_some, assert_some_eq};

    fn default_config() -> Config {
        assert_ok!(toml::from_str(include_str!("../../config/default.toml")))
    }

    fn default_tabs() -> Vec<Tab> {
        default_config().tabs.into_iter().map(Tab::new).collect()
    }

    #[test]
    fn required_fields_include_every_tab_column() {
        let tabs = default_tabs();
        let fields = required_fields(&tabs);

        for field in tabs.iter().flat_map(Tab::fields) {
            assert!(fields.contains(field), "missing {}", field.to_str());
        }
    }

    #[test]
    fn app_rejects_a_tab_with_an_unknown_status() {
        let mut config = default_config();
        config.tabs = vec![TabConfig {
            name: "Seeding".to_owned(),
            columns: vec!["name".to_owned()],
            statuses: vec!["seedng".to_owned()],
        }];

        let error = assert_err!(App::new(config));

        assert!(error.to_string().contains("seedng"), "{error}");
    }

    #[test]
    fn app_requests_fields_it_reads_outside_columns() {
        let mut config = default_config();
        config.tabs = vec![TabConfig {
            name: "Minimal".to_owned(),
            columns: vec!["eta".to_owned()],
            statuses: Vec::new(),
        }];
        let app = assert_ok!(App::new(config));
        let fields = assert_some!(app.torrents.fields.as_ref());

        // Listed independently of `APP_FIELDS`: selection and RPC actions need `id`,
        // filtering and rename need `name`, tab matching and row colors need `status`,
        // move prefill needs `downloadDir`, the Name column's progress underline needs
        // `percentDone`, and the status bar needs the totals.
        for field in [
            TorrentGetField::Id,
            TorrentGetField::Name,
            TorrentGetField::Status,
            TorrentGetField::DownloadDir,
            TorrentGetField::PercentDone,
            TorrentGetField::RateDownload,
            TorrentGetField::RateUpload,
            TorrentGetField::DownloadedEver,
            TorrentGetField::UploadedEver,
            TorrentGetField::Eta,
        ] {
            assert!(fields.contains(&field), "missing {}", field.to_str());
        }
    }

    #[test]
    fn required_fields_are_deduplicated() {
        let fields = required_fields(&default_tabs());

        for (idx, field) in fields.iter().enumerate() {
            assert!(
                !fields[idx + 1..].contains(field),
                "duplicate {}",
                field.to_str()
            );
        }
    }

    #[test]
    fn required_fields_exclude_unused_heavy_fields() {
        let fields = required_fields(&default_tabs());

        for heavy in [
            TorrentGetField::Files,
            TorrentGetField::FileStats,
            TorrentGetField::Peers,
            TorrentGetField::Pieces,
            TorrentGetField::TrackerStats,
            TorrentGetField::Trackers,
            TorrentGetField::Priorities,
            TorrentGetField::Wanted,
        ] {
            assert!(!fields.contains(&heavy), "unexpected {}", heavy.to_str());
        }
    }

    fn tab(columns: &[&str]) -> Tab {
        Tab::new(TabConfig {
            name: "Test".into(),
            columns: columns.iter().map(|&column| column.into()).collect(),
            statuses: Vec::new(),
        })
    }

    #[test]
    fn cycle_visits_each_column_then_daemon_order() {
        let mut tab = tab(&["name", "size", "eta"]);
        assert_none!(tab.sort());

        let mut visited = Vec::new();
        for _ in 0..3 {
            tab.cycle_sort();
            let sort = tab.sort().expect("cycling sorts by a column");
            visited.push((sort.column, sort.field, sort.direction));
        }
        assert_eq!(
            visited,
            [
                (0, TorrentGetField::Name, SortDirection::Ascending),
                (1, TorrentGetField::TotalSize, SortDirection::Descending),
                (2, TorrentGetField::Eta, SortDirection::Ascending),
            ]
        );

        tab.cycle_sort();
        assert_none!(tab.sort());
        tab.cycle_sort();
        assert_some_eq!(tab.sort(), Sort::new(0, TorrentGetField::Name));
    }

    #[test]
    fn reverse_flips_direction_and_cycle_resets_it() {
        let mut tab = tab(&["size", "name"]);
        tab.cycle_sort();

        tab.reverse_sort();
        assert_some_eq!(
            tab.sort().map(|sort| sort.direction),
            SortDirection::Ascending
        );
        tab.reverse_sort();
        assert_some_eq!(
            tab.sort().map(|sort| sort.direction),
            SortDirection::Descending
        );

        tab.reverse_sort();
        tab.cycle_sort();
        assert_some_eq!(tab.sort(), Sort::new(1, TorrentGetField::Name));
    }

    #[test]
    fn reverse_without_sort_keeps_daemon_order() {
        let mut tab = tab(&["name"]);
        tab.reverse_sort();
        assert_none!(tab.sort());
    }

    #[test]
    fn duplicate_columns_cycle_by_position() {
        let mut tab = tab(&["name", "name"]);
        tab.cycle_sort();
        tab.cycle_sort();
        assert_some_eq!(tab.sort().map(|sort| sort.column), 1);
        tab.cycle_sort();
        assert_none!(tab.sort());
    }
}
