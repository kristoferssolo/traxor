//! Column sorting for the torrent table.

use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use transmission_rpc::types::{Priority, Torrent, TorrentGetField};

/// Transmission's `TR_RATIO_INF`, reported when a torrent uploaded without downloading.
const RATIO_INFINITE: f32 = -2.0;

/// Order of sorted rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    /// Default direction for `field`: text, ETA, and positions ascend; amounts, rates, and
    /// dates descend so the largest or most recent come first.
    #[must_use]
    pub const fn natural(field: TorrentGetField) -> Self {
        match field {
            TorrentGetField::Comment
            | TorrentGetField::Creator
            | TorrentGetField::DownloadDir
            | TorrentGetField::Error
            | TorrentGetField::ErrorString
            | TorrentGetField::Eta
            | TorrentGetField::EtaIdle
            | TorrentGetField::Group
            | TorrentGetField::HashString
            | TorrentGetField::Id
            | TorrentGetField::Labels
            | TorrentGetField::MagnetLink
            | TorrentGetField::Name
            | TorrentGetField::PrimaryMimeType
            | TorrentGetField::QueuePosition
            | TorrentGetField::Status
            | TorrentGetField::TorrentFile
            | TorrentGetField::TrackerList => Self::Ascending,
            _ => Self::Descending,
        }
    }

    /// The opposite direction.
    #[must_use]
    pub const fn reversed(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    /// Header indicator for this direction.
    #[must_use]
    pub const fn arrow(self) -> &'static str {
        match self {
            Self::Ascending => "▲",
            Self::Descending => "▼",
        }
    }

    const fn apply(self, ordering: Ordering) -> Ordering {
        match self {
            Self::Ascending => ordering,
            Self::Descending => ordering.reverse(),
        }
    }
}

/// Sort of a tab's torrents by one of its columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    /// Index of the sorted column in the tab's columns.
    pub column: usize,
    pub field: TorrentGetField,
    pub direction: SortDirection,
}

impl Sort {
    /// Sort by `field`, shown in `column`, in its natural direction.
    #[must_use]
    pub const fn new(column: usize, field: TorrentGetField) -> Self {
        Self {
            column,
            field,
            direction: SortDirection::natural(field),
        }
    }

    /// The same column in the opposite direction.
    #[must_use]
    pub const fn reversed(self) -> Self {
        Self {
            direction: self.direction.reversed(),
            ..self
        }
    }

    /// Order `torrents` by the typed value of the sorted field.
    ///
    /// The sort is stable, so ties keep their input order. Torrents without a value for the
    /// field, or with a blank one such as an unknown ETA, come last in either direction.
    #[must_use]
    pub fn apply(self, torrents: Vec<&Torrent>) -> Vec<&Torrent> {
        let mut keyed = torrents
            .into_iter()
            .map(|torrent| (SortKey::of(self.field, torrent), torrent))
            .collect::<Vec<_>>();
        keyed.sort_by(|(a, _), (b, _)| match (a, b) {
            (Some(a), Some(b)) => self.direction.apply(a.compare(b)),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        });
        keyed.into_iter().map(|(_, torrent)| torrent).collect()
    }
}

/// Typed value a torrent is sorted by.
#[derive(Debug, Clone, PartialEq)]
enum SortKey {
    /// Lowercased text, so names sort case-insensitively.
    Text(String),
    /// Integers of any width, timestamps, and booleans.
    Integer(i128),
    Float(f32),
}

impl SortKey {
    /// Values of different variants never come from the same field, so they compare equal.
    fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a.cmp(b),
            (Self::Integer(a), Self::Integer(b)) => a.cmp(b),
            (Self::Float(a), Self::Float(b)) => a.total_cmp(b),
            _ => Ordering::Equal,
        }
    }

    /// Sort key of `field` for `torrent`, or [`None`] when the value is missing or blank.
    fn of(field: TorrentGetField, torrent: &Torrent) -> Option<Self> {
        match field {
            TorrentGetField::ActivityDate => date(torrent.activity_date),
            TorrentGetField::AddedDate => date(torrent.added_date),
            TorrentGetField::BandwidthPriority => torrent.bandwidth_priority.map(|priority| {
                Self::Integer(match priority {
                    Priority::Low => -1,
                    Priority::Normal => 0,
                    Priority::High => 1,
                })
            }),
            TorrentGetField::Comment => text(torrent.comment.as_deref()),
            TorrentGetField::CorruptEver => integer(torrent.corrupt_ever),
            TorrentGetField::Creator => text(torrent.creator.as_deref()),
            TorrentGetField::DateCreated => date(torrent.date_created),
            TorrentGetField::DesiredAvailable => integer(torrent.desired_available),
            TorrentGetField::DoneDate => date(torrent.done_date),
            TorrentGetField::DownloadDir => text(torrent.download_dir.as_deref()),
            TorrentGetField::DownloadLimit => integer(torrent.download_limit),
            TorrentGetField::DownloadLimited => integer(torrent.download_limited),
            TorrentGetField::DownloadedEver => integer(torrent.downloaded_ever),
            TorrentGetField::EditDate => date(torrent.edit_date),
            TorrentGetField::Error => torrent
                .error
                .map(|v| Self::Text(format!("{v:?}").to_lowercase())),
            TorrentGetField::ErrorString => text(torrent.error_string.as_deref()),
            TorrentGetField::Eta => eta(torrent.eta),
            TorrentGetField::EtaIdle => eta(torrent.eta_idle),
            TorrentGetField::FileCount => integer(torrent.file_count),
            TorrentGetField::Group => text(torrent.group.as_deref()),
            TorrentGetField::HashString => text(torrent.hash_string.as_deref()),
            TorrentGetField::HaveUnchecked => integer(torrent.have_unchecked),
            TorrentGetField::HaveValid => integer(torrent.have_valid),
            TorrentGetField::HonorsSessionLimits => integer(torrent.honors_session_limits),
            TorrentGetField::Id => integer(torrent.id),
            TorrentGetField::IsFinished => integer(torrent.is_finished),
            TorrentGetField::IsPrivate => integer(torrent.is_private),
            TorrentGetField::IsStalled => integer(torrent.is_stalled),
            TorrentGetField::Labels => {
                text(torrent.labels.as_ref().map(|l| l.join(", ")).as_deref())
            }
            TorrentGetField::LeftUntilDone => integer(torrent.left_until_done),
            TorrentGetField::MagnetLink => text(torrent.magnet_link.as_deref()),
            TorrentGetField::ManualAnnounceTime => date(torrent.manual_announce_time),
            TorrentGetField::MaxConnectedPeers => integer(torrent.max_connected_peers),
            TorrentGetField::MetadataPercentComplete => float(torrent.metadata_percent_complete),
            TorrentGetField::Name => text(torrent.name.as_deref()),
            TorrentGetField::PeerLimit => integer(torrent.peer_limit),
            TorrentGetField::PeersConnected => integer(torrent.peers_connected),
            TorrentGetField::PeersGettingFromUs => integer(torrent.peers_getting_from_us),
            TorrentGetField::PeersSendingToUs => integer(torrent.peers_sending_to_us),
            TorrentGetField::PercentComplete => float(torrent.percent_complete),
            TorrentGetField::PercentDone => float(torrent.percent_done),
            TorrentGetField::PieceCount => integer(torrent.piece_count),
            TorrentGetField::PieceSize => integer(torrent.piece_size),
            TorrentGetField::PrimaryMimeType => text(torrent.primary_mime_type.as_deref()),
            TorrentGetField::QueuePosition => integer(torrent.queue_position),
            TorrentGetField::RateDownload => integer(torrent.rate_download),
            TorrentGetField::RateUpload => integer(torrent.rate_upload),
            TorrentGetField::RecheckProgress => float(torrent.recheck_progress),
            TorrentGetField::SecondsDownloading => integer(torrent.seconds_downloading),
            TorrentGetField::SecondsSeeding => integer(torrent.seconds_seeding),
            TorrentGetField::SeedIdleLimit => integer(torrent.seed_idle_limit),
            TorrentGetField::SeedRatioLimit => float(torrent.seed_ratio_limit),
            TorrentGetField::SequentialDownload => integer(torrent.sequential_download),
            TorrentGetField::SizeWhenDone => integer(torrent.size_when_done),
            TorrentGetField::StartDate => date(torrent.start_date),
            TorrentGetField::Status => torrent
                .status
                .map(|v| Self::Text(format!("{v:?}").to_lowercase())),
            TorrentGetField::TorrentFile => text(torrent.torrent_file.as_deref()),
            TorrentGetField::TotalSize => integer(torrent.total_size),
            TorrentGetField::TrackerList => text(torrent.tracker_list.as_deref()),
            TorrentGetField::UploadLimit => integer(torrent.upload_limit),
            TorrentGetField::UploadLimited => integer(torrent.upload_limited),
            TorrentGetField::UploadRatio => ratio(torrent.upload_ratio),
            TorrentGetField::UploadedEver => integer(torrent.uploaded_ever),
            TorrentGetField::WebseedsSendingToUs => integer(torrent.webseeds_sending_to_us),
            // Lists and nested structures have no single value to order by.
            _ => None,
        }
    }
}

fn text(value: Option<&str>) -> Option<SortKey> {
    value
        .filter(|value| !value.is_empty())
        .map(|value| SortKey::Text(value.to_lowercase()))
}

fn integer<T: TryInto<i128>>(value: Option<T>) -> Option<SortKey> {
    value
        .and_then(|value| value.try_into().ok())
        .map(SortKey::Integer)
}

fn float(value: Option<f32>) -> Option<SortKey> {
    value.map(SortKey::Float)
}

/// Transmission reports unset dates as the Unix epoch, which the table shows blank.
fn date(value: Option<DateTime<Utc>>) -> Option<SortKey> {
    value
        .filter(|value| *value != DateTime::<Utc>::UNIX_EPOCH)
        .map(|value| SortKey::Integer(value.timestamp().into()))
}

/// Negative ETAs mean "not available" (-1) or "unknown" (-2).
fn eta(value: Option<i64>) -> Option<SortKey> {
    integer(value.filter(|value| *value >= 0))
}

/// Negative ratios mean "not available" (-1) or infinite (-2).
fn ratio(value: Option<f32>) -> Option<SortKey> {
    match value? {
        RATIO_INFINITE => Some(SortKey::Float(f32::INFINITY)),
        ratio if ratio < 0.0 => None,
        ratio => Some(SortKey::Float(ratio)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use claims::{assert_none, assert_ok, assert_some_eq};

    /// Torrent with `fields` set from TOML; every other field deserializes to [`None`].
    fn torrent(id: i64, fields: &str) -> Torrent {
        assert_ok!(toml::from_str(&format!("id = {id}\n{fields}")))
    }

    fn sorted_ids(torrents: &[Torrent], sort: Sort) -> Vec<i64> {
        sort.apply(torrents.iter().collect())
            .into_iter()
            .filter_map(|torrent| torrent.id)
            .collect()
    }

    fn sort(field: TorrentGetField) -> Sort {
        Sort::new(0, field)
    }

    #[test]
    fn sizes_sort_by_bytes_largest_first() {
        let torrents = [
            torrent(1, "totalSize = 900_000_000"),
            torrent(2, "totalSize = 2_000_000_000"),
            torrent(3, "totalSize = 1_024"),
        ];

        assert_eq!(
            sorted_ids(&torrents, sort(TorrentGetField::TotalSize)),
            [2, 1, 3]
        );
        assert_eq!(
            sorted_ids(&torrents, sort(TorrentGetField::TotalSize).reversed()),
            [3, 1, 2]
        );
    }

    #[test]
    fn eta_ascends_with_unknown_last_in_both_directions() {
        let torrents = [
            torrent(1, "eta = 300"),
            torrent(2, "eta = -1"),
            torrent(3, "eta = 60"),
            torrent(4, "eta = -2"),
            torrent(5, "eta = 0"),
        ];

        assert_eq!(
            sorted_ids(&torrents, sort(TorrentGetField::Eta)),
            [5, 3, 1, 2, 4]
        );
        assert_eq!(
            sorted_ids(&torrents, sort(TorrentGetField::Eta).reversed()),
            [1, 3, 5, 2, 4]
        );
    }

    #[test]
    fn names_sort_case_insensitively() {
        let torrents = [
            torrent(1, r#"name = "beta""#),
            torrent(2, r#"name = "Alpha""#),
            torrent(3, r#"name = "charlie""#),
            torrent(4, r#"name = "ALPHA""#),
        ];

        assert_eq!(
            sorted_ids(&torrents, sort(TorrentGetField::Name)),
            [2, 4, 1, 3]
        );
    }

    #[test]
    fn missing_values_sort_last_in_both_directions() {
        let torrents = [
            torrent(1, ""),
            torrent(2, "rateDownload = 10"),
            torrent(3, ""),
            torrent(4, "rateDownload = 500"),
        ];

        assert_eq!(
            sorted_ids(&torrents, sort(TorrentGetField::RateDownload)),
            [4, 2, 1, 3]
        );
        assert_eq!(
            sorted_ids(&torrents, sort(TorrentGetField::RateDownload).reversed()),
            [2, 4, 1, 3]
        );
    }

    #[test]
    fn ties_keep_input_order() {
        let torrents = [
            torrent(1, "uploadRatio = 1.5"),
            torrent(2, "uploadRatio = 0.5"),
            torrent(3, "uploadRatio = 1.5"),
            torrent(4, "uploadRatio = 0.5"),
        ];

        assert_eq!(
            sorted_ids(&torrents, sort(TorrentGetField::UploadRatio)),
            [1, 3, 2, 4]
        );
        assert_eq!(
            sorted_ids(&torrents, sort(TorrentGetField::UploadRatio).reversed()),
            [2, 4, 1, 3]
        );
    }

    #[test]
    fn infinite_ratio_sorts_highest_and_unavailable_last() {
        let torrents = [
            torrent(1, "uploadRatio = 3.0"),
            torrent(2, "uploadRatio = -1.0"),
            torrent(3, "uploadRatio = -2.0"),
        ];

        assert_eq!(
            sorted_ids(&torrents, sort(TorrentGetField::UploadRatio)),
            [3, 1, 2]
        );
    }

    #[test]
    fn dates_sort_by_timestamp_with_unset_last() {
        let key = |torrent: &Torrent| SortKey::of(TorrentGetField::AddedDate, torrent);
        let mut added = torrent(1, "");
        added.added_date = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).single();
        let mut unset = torrent(2, "");
        unset.added_date = Some(DateTime::<Utc>::UNIX_EPOCH);

        assert_some_eq!(key(&added), SortKey::Integer(1_704_164_645));
        assert_none!(key(&unset));
    }

    #[test]
    fn natural_directions() {
        for field in [
            TorrentGetField::Name,
            TorrentGetField::Status,
            TorrentGetField::Eta,
        ] {
            assert_eq!(SortDirection::natural(field), SortDirection::Ascending);
        }
        for field in [
            TorrentGetField::TotalSize,
            TorrentGetField::RateUpload,
            TorrentGetField::UploadRatio,
            TorrentGetField::PercentDone,
            TorrentGetField::AddedDate,
        ] {
            assert_eq!(SortDirection::natural(field), SortDirection::Descending);
        }
    }
}
