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
        Ok(Self {
            client: connect(rpc)?,
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

    /// Returns a [`TorrentFetcher`] with its own connection to `rpc` that requests the same
    /// fields as [`Self::update`].
    ///
    /// The fields are copied now, so later [`Self::set_fields`] calls do not affect it.
    ///
    /// # Errors
    ///
    /// Returns an error when the RPC URL or credentials are invalid.
    pub fn fetcher(&self, rpc: &RpcConfig) -> Result<TorrentFetcher> {
        Ok(TorrentFetcher {
            client: connect(rpc)?,
            fields: self.fields.clone(),
        })
    }

    /// Fetches torrents from the daemon, replacing the cached list.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn update(&mut self) -> Result<&mut Self> {
        self.torrents = fetch(&mut self.client, self.fields.clone()).await?;
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

/// Fetches the torrent list over its own RPC connection.
///
/// [`TransClient`] requests take `&mut self`, so sharing the client in [`Torrents`]
/// would hold the whole app for the duration of every refresh request. Built with
/// [`Torrents::fetcher`].
pub struct TorrentFetcher {
    client: TransClient,
    fields: Option<Vec<TorrentGetField>>,
}

impl TorrentFetcher {
    /// Requests the current torrent list.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC request fails.
    pub async fn fetch(&mut self) -> Result<Vec<Torrent>> {
        fetch(&mut self.client, self.fields.clone()).await
    }
}

/// Opens a client for the Transmission RPC endpoint in `rpc`.
///
/// Every client is built here so [`Torrents`] and [`TorrentFetcher`] always talk to
/// the same daemon with the same credentials.
fn connect(rpc: &RpcConfig) -> Result<TransClient> {
    let url = rpc.parsed_url()?;
    Ok(match rpc.basic_auth()? {
        Some(auth) => TransClient::with_auth(url, auth),
        None => TransClient::new(url),
    })
}

async fn fetch(
    client: &mut TransClient,
    fields: Option<Vec<TorrentGetField>>,
) -> Result<Vec<Torrent>> {
    Ok(client.torrent_get(fields, None).await?.arguments.torrents)
}
