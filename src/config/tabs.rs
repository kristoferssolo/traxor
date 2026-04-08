use serde::{Deserialize, Serialize};
use transmission_rpc::types::{TorrentGetField, TorrentStatus};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TabConfig {
    pub name: String,
    pub columns: Vec<String>,
    #[serde(default)]
    pub statuses: Vec<String>,
}

impl TabConfig {
    /// Parse column strings into `TorrentGetField` variants.
    #[must_use]
    pub fn fields(&self) -> Vec<TorrentGetField> {
        self.columns.iter().filter_map(|s| parse_field(s)).collect()
    }

    /// Parse status strings into `TorrentStatus` variants.
    #[must_use]
    pub fn statuses(&self) -> Vec<TorrentStatus> {
        self.statuses
            .iter()
            .filter_map(|status| parse_status(status))
            .collect()
    }
}

fn parse_field(s: &str) -> Option<TorrentGetField> {
    Some(match s.to_lowercase().as_str() {
        "name" => TorrentGetField::Name,
        "status" => TorrentGetField::Status,
        "size" | "totalsize" | "total_size" => TorrentGetField::TotalSize,
        "downloaded" | "downloadedever" | "downloaded_ever" => TorrentGetField::DownloadedEver,
        "uploaded" | "uploadedever" | "uploaded_ever" => TorrentGetField::UploadedEver,
        "ratio" | "uploadratio" | "upload_ratio" => TorrentGetField::UploadRatio,
        "progress" | "percent" | "percentdone" | "percent_done" => TorrentGetField::PercentDone,
        "eta" => TorrentGetField::Eta,
        "peers" | "peersconnected" | "peers_connected" => TorrentGetField::PeersConnected,
        "seeds" | "peerssending" | "peers_sending" => TorrentGetField::PeersSendingToUs,
        "leeches" | "peersgetting" | "peers_getting" => TorrentGetField::PeersGettingFromUs,
        "downspeed" | "ratedownload" | "rate_download" => TorrentGetField::RateDownload,
        "upspeed" | "rateupload" | "rate_upload" => TorrentGetField::RateUpload,
        "path" | "downloaddir" | "download_dir" => TorrentGetField::DownloadDir,
        "added" | "addeddate" | "added_date" => TorrentGetField::AddedDate,
        "done" | "donedate" | "done_date" => TorrentGetField::DoneDate,
        "left" | "leftuntildone" | "left_until_done" => TorrentGetField::LeftUntilDone,
        "queue" | "queueposition" | "queue_position" => TorrentGetField::QueuePosition,
        "error" => TorrentGetField::Error,
        "errorstring" | "error_string" => TorrentGetField::ErrorString,
        "labels" => TorrentGetField::Labels,
        "tracker" | "trackerlist" | "tracker_list" => TorrentGetField::TrackerList,
        "hash" | "hashstring" | "hash_string" => TorrentGetField::HashString,
        "private" | "isprivate" | "is_private" => TorrentGetField::IsPrivate,
        "stalled" | "isstalled" | "is_stalled" => TorrentGetField::IsStalled,
        "finished" | "isfinished" | "is_finished" => TorrentGetField::IsFinished,
        "files" | "filecount" | "file_count" => TorrentGetField::FileCount,
        "activity" | "activitydate" | "activity_date" => TorrentGetField::ActivityDate,
        _ => return None,
    })
}

fn parse_status(status: &str) -> Option<TorrentStatus> {
    Some(match status.to_lowercase().as_str() {
        "stopped" => TorrentStatus::Stopped,
        "queuedtoverify" | "queued_to_verify" | "queued-verify" => TorrentStatus::QueuedToVerify,
        "verifying" => TorrentStatus::Verifying,
        "queuedtodownload" | "queued_to_download" | "queued-download" => {
            TorrentStatus::QueuedToDownload
        }
        "downloading" => TorrentStatus::Downloading,
        "queuedtoseed" | "queued_to_seed" | "queued-seed" => TorrentStatus::QueuedToSeed,
        "seeding" => TorrentStatus::Seeding,
        _ => return None,
    })
}
