use transmission_rpc::types::TorrentGetField;

/// Available tabs.
/// TODO: write description
#[derive(Debug, Default)]
pub enum Tab {
    #[default]
    All,
    Active,
    Downloading,
    Settings,
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
                TorrentGetField::PercentDone,
                TorrentGetField::RateDownload,
                TorrentGetField::Eta,
                TorrentGetField::DownloadDir,
                TorrentGetField::Name,
            ],
            Tab::Settings => &[],
        }
    }
}

impl ToString for Tab {
    fn to_string(&self) -> String {
        match *self {
            Tab::All => String::from("All"),
            Tab::Active => String::from("Active"),
            Tab::Downloading => String::from("Downloading"),
            Tab::Settings => String::from("Settings"),
        }
    }
}
