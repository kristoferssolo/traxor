use transmission_rpc::types::{ErrorType, Torrent, TorrentGetField, TorrentStatus};

mod filesize;
mod netspeed;
use filesize::FileSize;

use self::netspeed::NetSpeed;

pub trait Wrapper {
    fn title(&self) -> String {
        String::from("")
    }

    fn value(&self, torrent: Torrent) -> String {
        format!("{}", torrent.name.unwrap_or(String::from("")))
    }

    fn width(&self) -> u16 {
        0
    }
}

impl Wrapper for TorrentGetField {
    fn title(&self) -> String {
        match self {
            TorrentGetField::ActivityDate => String::from("Activity Date"),
            TorrentGetField::AddedDate => String::from("Added Date"),
            TorrentGetField::DoneDate => String::from("Done Date"),
            TorrentGetField::DownloadDir => String::from("Path"),
            TorrentGetField::EditDate => String::from("Edit Date"),
            TorrentGetField::Error => String::from("Error Type"),
            TorrentGetField::ErrorString => String::from("Error String"),
            TorrentGetField::Eta => String::from("ETA"),
            TorrentGetField::FileStats => String::from("File Stats"),
            TorrentGetField::Files => String::from("Files"),
            TorrentGetField::HashString => String::from("Hash String"),
            TorrentGetField::Id => String::from("Id"),
            TorrentGetField::IsFinished => String::from("Finished"),
            TorrentGetField::IsPrivate => String::from("Private"),
            TorrentGetField::IsStalled => String::from("Stalled"),
            TorrentGetField::Labels => String::from("Labels"),
            TorrentGetField::LeftUntilDone => String::from("Left Until Done"),
            TorrentGetField::MetadataPercentComplete => String::from("Metadata Percent Complete"),
            TorrentGetField::Name => String::from("Name"),
            TorrentGetField::PeersConnected => String::from("Connected"),
            TorrentGetField::PeersGettingFromUs => String::from("Peers"),
            TorrentGetField::PeersSendingToUs => String::from("Seeds"),
            TorrentGetField::PercentDone => String::from("%"),
            TorrentGetField::Priorities => String::from("Priorities"),
            TorrentGetField::QueuePosition => String::from("Queue"),
            TorrentGetField::RateDownload => String::from("Download Speed"),
            TorrentGetField::RateUpload => String::from("Upload Speed"),
            TorrentGetField::RecheckProgress => String::from("Progress"),
            TorrentGetField::SecondsSeeding => String::from("Seconds Seeding"),
            TorrentGetField::SeedRatioLimit => String::from("Seed Ratio Limit"),
            TorrentGetField::SeedRatioMode => String::from("Seed Ratio Mode"),
            TorrentGetField::SizeWhenDone => String::from("Size"),
            TorrentGetField::Status => String::from("Status"),
            TorrentGetField::TorrentFile => String::from("Torrent File"),
            TorrentGetField::TotalSize => String::from("Total Size"),
            TorrentGetField::Trackers => String::from("Trackers"),
            TorrentGetField::UploadRatio => String::from("Ratio"),
            TorrentGetField::UploadedEver => String::from("Uploaded"),
            TorrentGetField::Wanted => String::from("Wanted"),
            TorrentGetField::WebseedsSendingToUs => String::from("Webseeds Sending to Us"),
            TorrentGetField::FileCount => unimplemented!(),
        }
    }

    fn value(&self, torrent: Torrent) -> String {
        match self {
            TorrentGetField::ActivityDate => optional_to_string(torrent.activity_date),
            TorrentGetField::AddedDate => optional_to_string(torrent.added_date),
            TorrentGetField::DoneDate => optional_to_string(torrent.done_date),
            TorrentGetField::DownloadDir => optional_to_string(torrent.download_dir),
            TorrentGetField::EditDate => optional_to_string(torrent.edit_date),
            TorrentGetField::Error => match torrent.error {
                Some(error) => match error {
                    ErrorType::Ok => String::from("Ok"),
                    ErrorType::LocalError => String::from("LocalError"),
                    ErrorType::TrackerError => String::from("TrackerError"),
                    ErrorType::TrackerWarning => String::from("TrackerWarning"),
                },
                None => String::from("N/A"),
            },
            TorrentGetField::ErrorString => optional_to_string(torrent.error_string),
            TorrentGetField::Eta => match torrent.eta {
                Some(eta) => match eta {
                    -1 => "".into(),
                    -2 => "?".into(),
                    _ => format!("{} s", eta),
                },
                None => String::from(""),
            },
            TorrentGetField::FileStats => match torrent.file_stats {
                Some(file_stats) => file_stats.iter().map(|x| x.priority.to_string()).collect(),
                None => String::from("N/A"),
            },
            TorrentGetField::Files => match torrent.files {
                Some(files) => files.iter().map(|x| x.name.to_owned()).collect(),
                None => String::from("N/A"),
            },
            TorrentGetField::HashString => optional_to_string(torrent.hash_string),
            TorrentGetField::Id => optional_to_string(torrent.id),
            TorrentGetField::IsFinished => optional_to_string(torrent.is_finished),
            TorrentGetField::IsPrivate => optional_to_string(torrent.is_private),
            TorrentGetField::IsStalled => optional_to_string(torrent.is_stalled),
            TorrentGetField::Labels => match torrent.labels {
                Some(labels) => labels.join(" "),
                None => String::from("N/A"),
            },
            TorrentGetField::LeftUntilDone => {
                FileSize(torrent.left_until_done.unwrap_or(0)).to_string()
            }
            TorrentGetField::MetadataPercentComplete => {
                optional_to_string(torrent.metadata_percent_complete)
            }
            TorrentGetField::Name => optional_to_string(torrent.name),
            TorrentGetField::PeersConnected => optional_to_string(torrent.peers_connected),
            TorrentGetField::PeersGettingFromUs => {
                optional_to_string(torrent.peers_getting_from_us)
            }
            TorrentGetField::PeersSendingToUs => optional_to_string(torrent.peers_sending_to_us),
            TorrentGetField::PercentDone => match torrent.percent_done {
                Some(percent_done) => format!("{:.0}", percent_done * 100.0),
                None => String::from("N/A"),
            },
            TorrentGetField::Priorities => match torrent.priorities {
                Some(priorities) => priorities.iter().map(|x| x.to_string()).collect(),
                None => String::from("N/A"),
            },
            TorrentGetField::QueuePosition => String::from("N/A"),
            TorrentGetField::RateDownload => {
                NetSpeed(torrent.rate_download.unwrap_or(0)).to_string()
            }
            TorrentGetField::RateUpload => NetSpeed(torrent.rate_upload.unwrap_or(0)).to_string(),
            TorrentGetField::RecheckProgress => optional_to_string(torrent.recheck_progress),
            TorrentGetField::SecondsSeeding => optional_to_string(torrent.seconds_seeding),
            TorrentGetField::SeedRatioLimit => optional_to_string(torrent.seed_ratio_limit),
            TorrentGetField::SeedRatioMode => String::from("N/A"),
            TorrentGetField::SizeWhenDone => {
                FileSize(torrent.size_when_done.unwrap_or(0)).to_string()
            }
            TorrentGetField::Status => match torrent.status {
                Some(status) => match status {
                    TorrentStatus::Stopped => String::from("Stopped"),
                    TorrentStatus::Seeding => String::from("Seeding"),
                    TorrentStatus::Verifying => String::from("Verifying"),
                    TorrentStatus::Downloading => String::from("Downloading"),
                    TorrentStatus::QueuedToSeed => String::from("QueuedToSeed"),
                    TorrentStatus::QueuedToVerify => String::from("QueuedToVerify"),
                    TorrentStatus::QueuedToDownload => String::from("QueuedToDownload"),
                },
                None => String::from("N/A"),
            },
            TorrentGetField::TorrentFile => optional_to_string(torrent.torrent_file),
            TorrentGetField::TotalSize => FileSize(torrent.total_size.unwrap_or(0)).to_string(),
            TorrentGetField::Trackers => match torrent.trackers {
                Some(trackers) => trackers.iter().map(|x| x.announce.to_string()).collect(),
                None => String::from("N/A"),
            },
            TorrentGetField::UploadRatio => match torrent.upload_ratio {
                Some(upload_ratio) => format!("{:.2}", upload_ratio),
                None => String::from("N/A"),
            },
            TorrentGetField::UploadedEver => {
                FileSize(torrent.uploaded_ever.unwrap_or(0)).to_string()
            }
            TorrentGetField::Wanted => match torrent.wanted {
                Some(wanted) => wanted.iter().map(|x| x.to_string()).collect(),
                None => String::from("N/A"),
            },
            TorrentGetField::WebseedsSendingToUs => String::from("N/A"),
            TorrentGetField::FileCount => unimplemented!(),
        }
    }

    fn width(&self) -> u16 {
        match self {
            TorrentGetField::ActivityDate => 10,
            TorrentGetField::AddedDate => 10,
            TorrentGetField::DoneDate => 10,
            TorrentGetField::DownloadDir => 30,
            TorrentGetField::EditDate => 10,
            TorrentGetField::Error => 10,
            TorrentGetField::ErrorString => 10,
            TorrentGetField::Eta => 10,
            TorrentGetField::FileStats => 10,
            TorrentGetField::Files => 10,
            TorrentGetField::HashString => 10,
            TorrentGetField::Id => 10,
            TorrentGetField::IsFinished => 10,
            TorrentGetField::IsPrivate => 10,
            TorrentGetField::IsStalled => 10,
            TorrentGetField::Labels => 10,
            TorrentGetField::LeftUntilDone => 10,
            TorrentGetField::MetadataPercentComplete => 10,
            TorrentGetField::Name => 70,
            TorrentGetField::PeersConnected => 10,
            TorrentGetField::PeersGettingFromUs => 10,
            TorrentGetField::PeersSendingToUs => 10,
            TorrentGetField::PercentDone => 10,
            TorrentGetField::Priorities => 10,
            TorrentGetField::QueuePosition => 10,
            TorrentGetField::RateDownload => 10,
            TorrentGetField::RateUpload => 10,
            TorrentGetField::RecheckProgress => 10,
            TorrentGetField::SecondsSeeding => 10,
            TorrentGetField::SeedRatioLimit => 10,
            TorrentGetField::SeedRatioMode => 10,
            TorrentGetField::SizeWhenDone => 10,
            TorrentGetField::Status => 15,
            TorrentGetField::TorrentFile => 10,
            TorrentGetField::TotalSize => 10,
            TorrentGetField::Trackers => 10,
            TorrentGetField::UploadRatio => 10,
            TorrentGetField::UploadedEver => 10,
            TorrentGetField::Wanted => 10,
            TorrentGetField::WebseedsSendingToUs => 10,
            TorrentGetField::FileCount => unimplemented!(),
        }
    }
}

fn optional_to_string<T: ToString>(option: Option<T>) -> String {
    option.map_or_else(|| "N/A".into(), |val| val.to_string())
}
