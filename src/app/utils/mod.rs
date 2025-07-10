pub mod filesize;
pub mod netspeed;
pub mod unit;

use filesize::FileSize;
use netspeed::NetSpeed;
use std::fmt::Display;
use transmission_rpc::types::{
    ErrorType, IdleMode, Priority, RatioMode, Torrent, TorrentGetField, TorrentStatus,
};

pub trait Wrapper {
    fn title(&self) -> String {
        String::new()
    }

    fn value(&self, torrent: &Torrent) -> String {
        torrent.name.clone().unwrap_or_default()
    }

    fn width(&self) -> u16 {
        0
    }
}

impl Wrapper for TorrentGetField {
    fn title(&self) -> String {
        match self {
            Self::ActivityDate => "Activity Date",
            Self::AddedDate => "Added Date",
            Self::Availability => "Availability",
            Self::BandwidthPriority => "Bandwidth Priority",
            Self::Comment => "Comment",
            Self::CorruptEver => "Corrupt Ever",
            Self::Creator => "Creator",
            Self::DateCreated => "Date Created",
            Self::DesiredAvailable => "Desired Available",
            Self::DoneDate => "Done Date",
            Self::DownloadDir => "Path",
            Self::DownloadLimit => "Download Limit",
            Self::DownloadLimited => "Download Limited",
            Self::DownloadedEver => "Downloaded Ever",
            Self::EditDate => "Edit Date",
            Self::Error => "Error Type",
            Self::ErrorString => "Error String",
            Self::Eta => "ETA",
            Self::EtaIdle => "ETA Idle",
            Self::FileCount => "File Count",
            Self::FileStats => "File Stats",
            Self::Files => "Files",
            Self::Group => "Group",
            Self::HashString => "Hash String",
            Self::HaveUnchecked => "Have Unchecked",
            Self::HaveValid => "Have Valid",
            Self::HonorsSessionLimits => "Honors Session Limits",
            Self::Id => "Id",
            Self::IsFinished => "Finished",
            Self::IsPrivate => "Private",
            Self::IsStalled => "Stalled",
            Self::Labels => "Labels",
            Self::LeftUntilDone => "Left Until Done",
            Self::MagnetLink => "Magnet Link",
            Self::ManualAnnounceTime => "Manual Announce Time",
            Self::MaxConnectedPeers => "Max Connected Peers",
            Self::MetadataPercentComplete => "Metadata Percent Complete",
            Self::Name => "Name",
            Self::PeerLimit => "Peer Limit",
            Self::Peers => "Peers",
            Self::PeersConnected => "Connected",
            Self::PeersFrom => "Peers From",
            Self::PeersGettingFromUs => "Peers Receiving",
            Self::PeersSendingToUs => "Seeds",
            Self::PercentComplete => "Percent Complete",
            Self::PercentDone => "%",
            Self::PieceCount => "Piece Count",
            Self::PieceSize => "Piece Size",
            Self::Pieces => "Pieces",
            Self::PrimaryMimeType => "Primary Mime Type",
            Self::Priorities => "Priorities",
            Self::QueuePosition => "Queue",
            Self::RateDownload => "Download Speed",
            Self::RateUpload => "Upload Speed",
            Self::RecheckProgress => "Progress",
            Self::SecondsDownloading => "Seconds Downloading",
            Self::SecondsSeeding => "Seconds Seeding",
            Self::SeedIdleLimit => "Seed Idle Limit",
            Self::SeedIdleMode => "Seed Idle Mode",
            Self::SeedRatioLimit => "Seed Ratio Limit",
            Self::SeedRatioMode => "Seed Ratio Mode",
            Self::SequentialDownload => "Sequential Download",
            Self::SizeWhenDone => "Size",
            Self::StartDate => "Start Date",
            Self::Status => "Status",
            Self::TorrentFile => "Torrent File",
            Self::TotalSize => "Total Size",
            Self::TrackerList => "Tracker List",
            Self::TrackerStats => "Tracker Stats",
            Self::Trackers => "Trackers",
            Self::UploadLimit => "Upload Limit",
            Self::UploadLimited => "Upload Limited",
            Self::UploadRatio => "Ratio",
            Self::UploadedEver => "Uploaded",
            Self::Wanted => "Wanted",
            Self::Webseeds => "Webseeds",
            Self::WebseedsSendingToUs => "Webseeds Sending to Us",
        }
        .into()
    }

    fn value(&self, torrent: &Torrent) -> String {
        match self {
            Self::ActivityDate => format_option_string(torrent.activity_date),
            Self::AddedDate => format_option_string(torrent.added_date),
            Self::Availability => "N/A".to_string(),
            Self::BandwidthPriority => torrent.bandwidth_priority.format(),
            Self::Comment => torrent.comment.clone().unwrap_or_default(),
            Self::CorruptEver => FileSize::from(torrent.corrupt_ever).to_string(),
            Self::Creator => torrent.creator.clone().unwrap_or_default(),
            Self::DateCreated => format_option_string(torrent.date_created),
            Self::DesiredAvailable => FileSize::from(torrent.desired_available).to_string(),
            Self::DoneDate => format_option_string(torrent.done_date),
            Self::DownloadDir => torrent.download_dir.clone().unwrap_or_default(),
            Self::DownloadLimit => NetSpeed::from(torrent.download_limit).to_string(),
            Self::DownloadLimited => format_option_string(torrent.download_limited),
            Self::DownloadedEver => FileSize::from(torrent.downloaded_ever).to_string(),
            Self::EditDate => format_option_string(torrent.edit_date),
            Self::Error => torrent.error.format(),
            Self::ErrorString => torrent.error_string.clone().unwrap_or_default(),
            Self::Eta => format_eta(torrent.eta),
            Self::EtaIdle => format_option_string(torrent.eta_idle),
            Self::FileCount => format_option_string(torrent.file_count),
            Self::FileStats => torrent.file_stats.format(),
            Self::Files => torrent.files.format(),
            Self::Group => torrent.group.clone().unwrap_or_default(),
            Self::HashString => torrent.hash_string.clone().unwrap_or_default(),
            Self::HaveUnchecked => FileSize::from(torrent.have_unchecked).to_string(),
            Self::HaveValid => FileSize::from(torrent.have_valid).to_string(),
            Self::HonorsSessionLimits => format_option_string(torrent.honors_session_limits),
            Self::Id => format_option_string(torrent.id),
            Self::IsFinished => format_option_string(torrent.is_finished),
            Self::IsPrivate => format_option_string(torrent.is_private),
            Self::IsStalled => format_option_string(torrent.is_stalled),
            Self::Labels => torrent.labels.clone().unwrap_or_default().join(", "),
            Self::LeftUntilDone => FileSize::from(torrent.left_until_done).to_string(),
            Self::MagnetLink => torrent.magnet_link.clone().unwrap_or_default(),
            Self::ManualAnnounceTime => format_option_string(torrent.manual_announce_time),
            Self::MaxConnectedPeers => format_option_string(torrent.max_connected_peers),
            Self::MetadataPercentComplete => torrent.metadata_percent_complete.format(),
            Self::Name => torrent.name.clone().unwrap_or_default(),
            Self::PeerLimit => format_option_string(torrent.peer_limit),
            Self::Peers => torrent.peers.format(),
            Self::PeersConnected => format_option_string(torrent.peers_connected),
            Self::PeersFrom => torrent
                .peers_from
                .as_ref()
                .map(|p| {
                    format!(
                        "d:{} u:{} i:{} t:{}",
                        p.from_dht, p.from_incoming, p.from_lpd, p.from_tracker
                    )
                })
                .unwrap_or_default(),
            Self::PeersGettingFromUs => format_option_string(torrent.peers_getting_from_us),
            Self::PeersSendingToUs => format_option_string(torrent.peers_sending_to_us),
            Self::PercentComplete => torrent.percent_complete.format(),
            Self::PercentDone => torrent.percent_done.format(),
            Self::PieceCount => format_option_string(torrent.piece_count),
            Self::PieceSize => FileSize::from(torrent.piece_size).to_string(),
            Self::Pieces => torrent
                .pieces
                .as_ref()
                .map(|p| format!("{} bytes", p.len()))
                .unwrap_or_default(),
            Self::PrimaryMimeType => torrent.primary_mime_type.clone().unwrap_or_default(),
            Self::Priorities => torrent.priorities.format(),
            Self::QueuePosition => format_option_string(torrent.queue_position),
            Self::RateDownload => NetSpeed::from(torrent.rate_download).to_string(),
            Self::RateUpload => NetSpeed::from(torrent.rate_upload).to_string(),
            Self::RecheckProgress => torrent.recheck_progress.format(),
            Self::SecondsDownloading => format_option_string(torrent.seconds_downloading),
            Self::SecondsSeeding => format_option_string(torrent.seconds_seeding),
            Self::SeedIdleLimit => format_option_string(torrent.seed_idle_limit),
            Self::SeedIdleMode => torrent.seed_idle_mode.format(),
            Self::SeedRatioLimit => torrent.seed_ratio_limit.format(),
            Self::SeedRatioMode => torrent.seed_ratio_mode.format(),
            Self::SequentialDownload => format_option_string(torrent.sequential_download),
            Self::SizeWhenDone => FileSize::from(torrent.size_when_done).to_string(),
            Self::StartDate => format_option_string(torrent.start_date),
            Self::Status => torrent.status.format(),
            Self::TorrentFile => torrent.torrent_file.clone().unwrap_or_default(),
            Self::TotalSize => FileSize::from(torrent.total_size).to_string(),
            Self::TrackerList => torrent.tracker_list.clone().unwrap_or_default(),
            Self::TrackerStats => torrent.tracker_stats.format(),
            Self::Trackers => torrent.trackers.format(),
            Self::UploadLimit => NetSpeed::from(torrent.upload_limit).to_string(),
            Self::UploadLimited => format_option_string(torrent.upload_limited),
            Self::UploadRatio => torrent.upload_ratio.format(),
            Self::UploadedEver => FileSize::from(torrent.uploaded_ever).to_string(),
            Self::Wanted => torrent.wanted.format(),
            Self::Webseeds => torrent.webseeds.clone().unwrap_or_default().join(", "),
            Self::WebseedsSendingToUs => format_option_string(torrent.webseeds_sending_to_us),
        }
    }

    fn width(&self) -> u16 {
        #![allow(clippy::match_same_arms)]
        match self {
            Self::ActivityDate => 20,
            Self::AddedDate => 20,
            Self::Availability => 10,
            Self::BandwidthPriority => 10,
            Self::Comment => 20,
            Self::CorruptEver => 15,
            Self::Creator => 20,
            Self::DateCreated => 20,
            Self::DesiredAvailable => 15,
            Self::DoneDate => 20,
            Self::DownloadDir => 30,
            Self::DownloadLimit => 15,
            Self::DownloadLimited => 10,
            Self::DownloadedEver => 15,
            Self::EditDate => 20,
            Self::Error => 15,
            Self::ErrorString => 20,
            Self::Eta => 10,
            Self::EtaIdle => 10,
            Self::FileCount => 10,
            Self::FileStats => 10,
            Self::Files => 10,
            Self::Group => 10,
            Self::HashString => 42,
            Self::HaveUnchecked => 15,
            Self::HaveValid => 15,
            Self::HonorsSessionLimits => 10,
            Self::Id => 5,
            Self::IsFinished => 10,
            Self::IsPrivate => 10,
            Self::IsStalled => 10,
            Self::Labels => 20,
            Self::LeftUntilDone => 15,
            Self::MagnetLink => 50,
            Self::ManualAnnounceTime => 20,
            Self::MaxConnectedPeers => 10,
            Self::MetadataPercentComplete => 10,
            Self::Name => 70,
            Self::PeerLimit => 10,
            Self::Peers => 10,
            Self::PeersConnected => 10,
            Self::PeersFrom => 20,
            Self::PeersGettingFromUs => 10,
            Self::PeersSendingToUs => 10,
            Self::PercentComplete => 10,
            Self::PercentDone => 10,
            Self::PieceCount => 10,
            Self::PieceSize => 15,
            Self::Pieces => 20,
            Self::PrimaryMimeType => 20,
            Self::Priorities => 10,
            Self::QueuePosition => 10,
            Self::RateDownload => 15,
            Self::RateUpload => 15,
            Self::RecheckProgress => 10,
            Self::SecondsDownloading => 15,
            Self::SecondsSeeding => 15,
            Self::SeedIdleLimit => 10,
            Self::SeedIdleMode => 15,
            Self::SeedRatioLimit => 10,
            Self::SeedRatioMode => 15,
            Self::SequentialDownload => 10,
            Self::SizeWhenDone => 15,
            Self::StartDate => 20,
            Self::Status => 15,
            Self::TorrentFile => 30,
            Self::TotalSize => 15,
            Self::TrackerList => 30,
            Self::TrackerStats => 10,
            Self::Trackers => 10,
            Self::UploadLimit => 15,
            Self::UploadLimited => 10,
            Self::UploadRatio => 10,
            Self::UploadedEver => 15,
            Self::Wanted => 10,
            Self::Webseeds => 20,
            Self::WebseedsSendingToUs => 10,
        }
    }
}

fn format_option_string<T: Display>(value: Option<T>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn format_eta(value: Option<i64>) -> String {
    match value {
        Some(-2) => "?".into(),
        None | Some(-1 | ..0) => String::new(),
        Some(v) => format!("{v} s"),
    }
}

trait Formatter {
    fn format(&self) -> String;
}

impl Formatter for Option<f32> {
    fn format(&self) -> String {
        self.map(|v| format!("{v:.2}")).unwrap_or_default()
    }
}

impl<T> Formatter for Option<Vec<T>> {
    fn format(&self) -> String {
        self.as_ref()
            .map(|v| v.len().to_string())
            .unwrap_or_default()
    }
}

macro_rules! impl_enum_formatter {
    ($enum_type:ty, { $($variant:pat => $str:expr),* $(,)? }) => {
        impl Formatter for Option<$enum_type> {
            fn format(&self) -> String {
                self.map_or("N/A".to_string(),|v| match v { $($variant => $str,)* }.to_string())
            }
        }
    };
}

impl_enum_formatter!(Priority, {
    Priority::Low => "Low",
    Priority::Normal => "Normal",
    Priority::High => "High",
});

impl_enum_formatter!(IdleMode, {
    IdleMode::Global => "Global",
    IdleMode::Single => "Single",
    IdleMode::Unlimited => "Unlimited",
});

impl_enum_formatter!(RatioMode, {
    RatioMode::Global => "Global",
    RatioMode::Single => "Single",
    RatioMode::Unlimited => "Unlimited",
});

impl_enum_formatter!(TorrentStatus, {
    TorrentStatus::Stopped => "Stopped",
    TorrentStatus::Seeding => "Seeding",
    TorrentStatus::Verifying => "Verifying",
    TorrentStatus::Downloading => "Downloading",
    TorrentStatus::QueuedToSeed => "QueuedToSeed",
    TorrentStatus::QueuedToVerify => "QueuedToVerify",
    TorrentStatus::QueuedToDownload => "QueuedToDownload",
});

impl_enum_formatter!(ErrorType, {
    ErrorType::Ok => "OK",
    ErrorType::TrackerWarning => "TrackerWarning",
    ErrorType::TrackerError => "TrackerError",
    ErrorType::LocalError => "LocalError",
});
