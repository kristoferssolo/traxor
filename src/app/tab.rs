use std::fmt::Display;
use transmission_rpc::types::TorrentGetField;

/// Available tabs.
#[derive(Debug, Clone, Default)]
pub enum Tab {
    #[default]
    All,
    Active,
    Downloading,
}

impl Tab {
    /// Returns slice [`TorrentGetField`] apropriate variants.
    #[must_use]
    pub const fn fields(&self) -> &[TorrentGetField] {
        match self {
            Self::All => &[
                TorrentGetField::Status,
                TorrentGetField::PeersGettingFromUs,
                TorrentGetField::UploadRatio,
                TorrentGetField::TotalSize,
                TorrentGetField::UploadedEver,
                TorrentGetField::DownloadDir,
                TorrentGetField::Name,
            ],
            Self::Active => &[
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
            Self::Downloading => &[
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

impl From<usize> for Tab {
    fn from(value: usize) -> Self {
        #[allow(clippy::match_same_arms)]
        match value {
            0 => Self::All,
            1 => Self::Active,
            2 => Self::Downloading,
            _ => Self::All,
        }
    }
}

impl AsRef<str> for Tab {
    fn as_ref(&self) -> &str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Downloading => "Downloading",
        }
    }
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
