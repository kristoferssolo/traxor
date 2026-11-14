use crate::{config::rpc::RpcConfig, error::Result};
use std::{collections::HashSet, fmt::Debug};
use transmission_rpc::{
    TransClient,
    types::{Torrent, TorrentGetField},
};

/// List of torrents.
pub struct Torrents {
    /// Transmission RPC client.
    pub client: TransClient,
    pub torrents: Vec<Torrent>,
    pub selected: HashSet<i64>,
    pub fields: Option<Vec<TorrentGetField>>,
}

impl Torrents {
    /// Constructs a new instance of [`Torrents`] with a client for `rpc`.
    ///
    /// # Errors
    ///
    /// Returns an error when the RPC URL or credentials are invalid.
    pub fn new(rpc: &RpcConfig) -> Result<Self> {
        let url = rpc.parsed_url()?;
        let client = match rpc.basic_auth()? {
            Some(auth) => TransClient::with_auth(url, auth),
            None => TransClient::new(url),
        };
        Ok(Self {
            client,
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
