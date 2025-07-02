pub mod filesize;
pub mod netspeed;

use filesize::FileSize;
use netspeed::NetSpeed;
use transmission_rpc::types::{
    ErrorType, IdleMode, RatioMode, Torrent, TorrentGetField, TorrentStatus,
};

pub trait Wrapper {
    fn title(&self) -> String {
        "".to_string()
    }

    fn value(&self, torrent: &Torrent) -> String {
        format!("{}", torrent.name.as_ref().unwrap_or(&String::from("")))
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
            Self::PeersGettingFromUs => "Peers",
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
            Self::ActivityDate => torrent
                .activity_date
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::AddedDate => torrent
                .added_date
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::Availability => "N/A".to_string(),
            Self::BandwidthPriority => torrent
                .bandwidth_priority
                .map(|v| format!("{:?}", v))
                .unwrap_or_default(),
            Self::Comment => torrent.comment.clone().unwrap_or_default(),
            Self::CorruptEver => FileSize::from(torrent.corrupt_ever.unwrap_or(0)).to_string(),
            Self::Creator => torrent.creator.clone().unwrap_or_default(),
            Self::DateCreated => torrent
                .date_created
                .map(|v| v.to_string())
                .unwrap_or_default()
                .to_string(),
            Self::DesiredAvailable => {
                FileSize::from(torrent.desired_available.unwrap_or(0)).to_string()
            }
            Self::DoneDate => torrent.done_date.map(|v| v.to_string()).unwrap_or_default(),
            Self::DownloadDir => torrent.download_dir.clone().unwrap_or_default().to_string(),
            Self::DownloadLimit => NetSpeed::from(torrent.download_limit.unwrap_or(0)).to_string(),
            Self::DownloadLimited => torrent
                .download_limited
                .map(|v| v.to_string())
                .unwrap_or_default()
                .to_string(),
            Self::DownloadedEver => {
                FileSize::from(torrent.downloaded_ever.unwrap_or(0)).to_string()
            }
            Self::EditDate => torrent.edit_date.map(|v| v.to_string()).unwrap_or_default(),
            Self::Error => match torrent.error {
                Some(error) => match error {
                    ErrorType::Ok => "Ok",
                    ErrorType::LocalError => "LocalError",
                    ErrorType::TrackerError => "TrackerError",
                    ErrorType::TrackerWarning => "TrackerWarning",
                },
                None => "N/A",
            }
            .to_string(),
            Self::ErrorString => torrent.error_string.clone().unwrap_or_default(),
            Self::Eta => match torrent.eta {
                Some(eta) => match eta {
                    -1 => "".to_string(),
                    -2 => "?".to_string(),
                    _ => format!("{} s", eta),
                },
                None => "".to_string(),
            },
            Self::EtaIdle => torrent.eta_idle.map(|v| v.to_string()).unwrap_or_default(),
            Self::FileCount => torrent
                .file_count
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::FileStats => torrent
                .file_stats
                .as_ref()
                .map(|v| format!("{}", v.len()))
                .unwrap_or_default(),
            Self::Files => torrent
                .files
                .as_ref()
                .map(|v| format!("{}", v.len()))
                .unwrap_or_default(),
            Self::Group => torrent.group.clone().unwrap_or_default(),
            Self::HashString => torrent.hash_string.clone().unwrap_or_default(),
            Self::HaveUnchecked => todo!(),
            Self::HaveValid => FileSize::from(torrent.have_valid.unwrap_or(0)).to_string(),
            Self::HonorsSessionLimits => torrent
                .honors_session_limits
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::Id => torrent.id.map(|v| v.to_string()).unwrap_or_default(),
            Self::IsFinished => torrent
                .is_finished
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::IsPrivate => torrent
                .is_private
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::IsStalled => torrent
                .is_stalled
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::Labels => torrent.labels.clone().unwrap_or_default().join(", "),
            Self::LeftUntilDone => todo!(),
            Self::MagnetLink => torrent.magnet_link.clone().unwrap_or_default(),
            Self::ManualAnnounceTime => torrent
                .manual_announce_time
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::MaxConnectedPeers => torrent
                .max_connected_peers
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::MetadataPercentComplete => torrent
                .metadata_percent_complete
                .map(|v| format!("{:.2}", v))
                .unwrap_or_default(),
            Self::Name => torrent.name.clone().unwrap_or_default(),
            Self::PeerLimit => torrent
                .peer_limit
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::Peers => torrent
                .peers
                .as_ref()
                .map(|v| format!("{}", v.len()))
                .unwrap_or_default(),
            Self::PeersConnected => torrent
                .peers_connected
                .map(|v| v.to_string())
                .unwrap_or_default(),
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
            Self::PeersGettingFromUs => torrent
                .peers_getting_from_us
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::PeersSendingToUs => torrent
                .peers_sending_to_us
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::PercentComplete => torrent
                .percent_complete
                .map(|v| format!("{:.2}", v))
                .unwrap_or_default(),
            Self::PercentDone => torrent
                .percent_done
                .map(|v| format!("{:.2}", v))
                .unwrap_or_default(),
            Self::PieceCount => torrent
                .piece_count
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::PieceSize => FileSize::from(torrent.piece_size.unwrap_or(0)).to_string(),
            Self::Pieces => torrent
                .pieces
                .as_ref()
                .map(|p| format!("{} bytes", p.len()))
                .unwrap_or_default(),
            Self::PrimaryMimeType => torrent.primary_mime_type.clone().unwrap_or_default(),
            Self::Priorities => torrent
                .priorities
                .as_ref()
                .map(|v| format!("{}", v.len()))
                .unwrap_or_default(),
            Self::QueuePosition => torrent
                .queue_position
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::RateDownload => NetSpeed::try_from(torrent.rate_download.unwrap_or(0))
                .unwrap_or_default()
                .to_string(),
            Self::RateUpload => NetSpeed::try_from(torrent.rate_upload.unwrap_or(0))
                .unwrap_or_default()
                .to_string(),
            Self::RecheckProgress => torrent
                .recheck_progress
                .map(|v| format!("{:.2}", v))
                .unwrap_or_default(),
            Self::SecondsDownloading => torrent
                .seconds_downloading
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::SecondsSeeding => torrent
                .seconds_seeding
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::SeedIdleLimit => torrent
                .seed_idle_limit
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::SeedIdleMode => torrent
                .seed_idle_mode
                .map(|v| match v {
                    IdleMode::Global => "Global",
                    IdleMode::Single => "Single",
                    IdleMode::Unlimited => "Unlimited",
                })
                .unwrap_or("N/A")
                .to_string(),
            Self::SeedRatioLimit => torrent
                .seed_ratio_limit
                .map(|v| format!("{:.2}", v))
                .unwrap_or_default(),
            Self::SeedRatioMode => torrent
                .seed_ratio_mode
                .map(|v| match v {
                    RatioMode::Global => "Global",
                    RatioMode::Single => "Single",
                    RatioMode::Unlimited => "Unlimited",
                })
                .unwrap_or_default()
                .to_string(),
            Self::SequentialDownload => torrent
                .sequential_download
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::SizeWhenDone => FileSize::try_from(torrent.size_when_done.unwrap_or(0))
                .unwrap_or_default()
                .to_string(),
            Self::StartDate => torrent
                .start_date
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::Status => match torrent.status {
                Some(status) => match status {
                    TorrentStatus::Stopped => "Stopped",
                    TorrentStatus::Seeding => "Seeding",
                    TorrentStatus::Verifying => "Verifying",
                    TorrentStatus::Downloading => "Downloading",
                    TorrentStatus::QueuedToSeed => "QueuedToSeed",
                    TorrentStatus::QueuedToVerify => "QueuedToVerify",
                    TorrentStatus::QueuedToDownload => "QueuedToDownload",
                },
                None => "N/A",
            }
            .to_string(),
            Self::TorrentFile => torrent.torrent_file.clone().unwrap_or_default(),
            Self::TotalSize => FileSize::try_from(torrent.total_size.unwrap_or(0))
                .unwrap_or_default()
                .to_string(),
            Self::TrackerList => torrent.tracker_list.clone().unwrap_or_default(),
            Self::TrackerStats => torrent
                .tracker_stats
                .as_ref()
                .map(|v| format!("{}", v.len()))
                .unwrap_or_default(),
            Self::Trackers => torrent
                .trackers
                .as_ref()
                .map(|v| format!("{}", v.len()))
                .unwrap_or_default(),
            Self::UploadLimit => NetSpeed::try_from(torrent.upload_limit.unwrap_or(0))
                .unwrap_or_default()
                .to_string(),
            Self::UploadLimited => torrent
                .upload_limited
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Self::UploadRatio => torrent
                .upload_ratio
                .map(|v| format!("{:.2}", v))
                .unwrap_or_default(),
            Self::UploadedEver => FileSize::try_from(torrent.uploaded_ever.unwrap_or(0))
                .unwrap_or_default()
                .to_string(),
            Self::Wanted => torrent
                .wanted
                .as_ref()
                .map(|v| format!("{}", v.len()))
                .unwrap_or_default(),
            Self::Webseeds => torrent.webseeds.clone().unwrap_or_default().join(", "),
            Self::WebseedsSendingToUs => torrent
                .webseeds_sending_to_us
                .map(|v| v.to_string())
                .unwrap_or_default(),
        }
    }

    fn width(&self) -> u16 {
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
