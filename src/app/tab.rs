use transmission_rpc::types::TorrentGetField;

/// Available tabs.
#[derive(Debug, Default)]
pub enum Tab {
    #[default]
    All,
    Active,
    Downloading,
}

impl Tab {
    /// Returns slice [`TorrentGetField`] apropriate variants.
    pub fn fields(&self) -> &[TorrentGetField] {
        match self {
            Tab::All => &[
                TorrentGetField::Status,
                TorrentGetField::PeersGettingFromUs,
                TorrentGetField::UploadRatio,
                TorrentGetField::TotalSize,
                TorrentGetField::UploadedEver,
                TorrentGetField::DownloadDir,
                TorrentGetField::Name,
            ],
            Tab::Active => &[
                TorrentGetField::TotalSize,
                TorrentGetField::UploadedEver,
                TorrentGetField::UploadRatio,
                TorrentGetField::PeersGettingFromUs,
                TorrentGetField::PeersSendingToUs,
                TorrentGetField::Status,
                TorrentGetField::Eta,
                TorrentGetField::PercentDone,
                TorrentGetField::RateDownload,
                TorrentGetField::RateUpload,
                TorrentGetField::Name,
            ],
            Tab::Downloading => &[
                TorrentGetField::TotalSize,
                TorrentGetField::LeftUntilDone,
                TorrentGetField::PercentDone,
                TorrentGetField::RateDownload,
                TorrentGetField::Eta,
                TorrentGetField::DownloadDir,
                TorrentGetField::Name,
            ],
        }
    }
}

impl AsRef<str> for Tab {
    fn as_ref(&self) -> &str {
        match self {
            Tab::All => "All",
            Tab::Active => "Active",
            Tab::Downloading => "Downloading",
        }
    }
}

impl ToString for Tab {
    fn to_string(&self) -> String {
        self.as_ref().into()
    }
}
