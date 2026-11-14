use crate::{app::constants::DEFAULT_RPC_URL, error::Result};
use std::{collections::HashSet, fmt::Debug};
use transmission_rpc::{
    TransClient,
    types::{Torrent, TorrentGetField},
};
use url::Url;

/// List of torrents.
pub struct Torrents {
    /// Transmission RPC client.
    pub client: TransClient,
    pub torrents: Vec<Torrent>,
    pub selected: HashSet<i64>,
    pub fields: Option<Vec<TorrentGetField>>,
}

impl Torrents {
    /// Constructs a new instance of [`Torrents`] for the default RPC URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the default RPC URL is invalid.
    pub fn new() -> Result<Self> {
        let url = Url::parse(DEFAULT_RPC_URL)?;
        Ok(Self {
            client: TransClient::new(url),
            torrents: Vec::new(),
            selected: HashSet::new(),
            fields: None,
        })
    }

    /// Returns the number of [`Torrent`]s in [`Torrents`]
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.torrents.len()
    }

    /// Returns `true` if the `torrents` contains no elements.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.torrents.is_empty()
    }

    /// Sets the fields that [`Self::update`] requests.
    pub fn set_fields(&mut self, fields: Option<Vec<TorrentGetField>>) -> &mut Self {
        self.fields = fields;
        self
    }

    /// Replaces the client with one configured for `url`.
    ///
    /// # Errors
    ///
    /// Returns an error if `url` is invalid.
    pub fn url(&mut self, url: &str) -> Result<&mut Self> {
        self.client = TransClient::new(Url::parse(url)?);
        Ok(self)
    }

    /// Fetches torrents from the daemon, replacing the cached list.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn update(&mut self) -> Result<&mut Self> {
        self.torrents = self
            .client
            .torrent_get(self.fields.clone(), None)
            .await?
            .arguments
            .torrents;
        Ok(self)
    }
}

impl Debug for Torrents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fields = self.fields.as_ref().map_or_else(
            || vec!["None".into()],
            |fields| fields.iter().map(TorrentGetField::to_str).collect(),
        );
        write!(
            f,
            "fields:
{:?};

torrents: {:?}",
            fields, self.torrents
        )
    }
}
