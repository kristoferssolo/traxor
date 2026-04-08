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
    /// Validate configured tab columns and statuses.
    ///
    /// # Errors
    ///
    /// Returns an error when the tab contains unknown column or status names.
    pub fn validate(&self, idx: usize) -> Result<(), String> {
        let invalid_columns = self
            .columns
            .iter()
            .filter(|column| parse_field(column).is_none())
            .cloned()
            .collect::<Vec<_>>();
        if !invalid_columns.is_empty() {
            return Err(format!(
                "invalid columns in tabs[{idx}] ({:?}): {}",
                self.name,
                invalid_columns.join(", ")
            ));
        }

        let invalid_statuses = self
            .statuses
            .iter()
            .filter(|status| parse_status(status).is_none())
            .cloned()
            .collect::<Vec<_>>();
        if !invalid_statuses.is_empty() {
            return Err(format!(
                "invalid statuses in tabs[{idx}] ({:?}): {}",
                self.name,
                invalid_statuses.join(", ")
            ));
        }

        Ok(())
    }

    /// Parse column strings into `TorrentGetField` variants.
    ///
    /// # Panics
    ///
    /// Panics if called before [`Self::validate`] succeeds for this tab.
    #[must_use]
    pub fn fields(&self) -> Vec<TorrentGetField> {
        self.columns
            .iter()
            .map(|column| parse_field(column).expect("tab columns should be validated before use"))
            .collect()
    }

    /// Parse status strings into `TorrentStatus` variants.
    ///
    /// # Panics
    ///
    /// Panics if called before [`Self::validate`] succeeds for this tab.
    #[must_use]
    pub fn statuses(&self) -> Vec<TorrentStatus> {
        self.statuses
            .iter()
            .map(|status| {
                parse_status(status).expect("tab statuses should be validated before use")
            })
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
