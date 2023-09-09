use std::fmt::Debug;

use anyhow::Result;
use transmission_rpc::{
    types::{Id, Torrent, TorrentGetField},
    TransClient,
};

use url::Url;

/// List of torrents.
pub struct Torrents {
    /// Constructs a new instance of [`Torrents`].
    client: TransClient,
    torrents: Vec<Torrent>,
    ids: Option<Vec<Id>>,
    fields: Option<Vec<TorrentGetField>>,
}

impl Torrents {
    /// Constructs a new instance of [`Torrents`].
    pub fn new() -> Torrents {
        let url = Url::parse("http://localhost:9091/transmission/rpc").unwrap();
        Self {
            client: TransClient::new(url),
            torrents: Vec::new(),
            ids: None,
            fields: None,
        }
    }

    /// Returns the number of [`Torrent`]s in [`Torrents`]
    pub fn len(&self) -> usize {
        self.torrents.len()
    }

    /// Sets `self.fields`
    pub fn set_fields(&mut self, fields: Option<Vec<TorrentGetField>>) -> &mut Self {
        self.fields = fields;
        self
    }

    /// Sets
    pub fn url(&mut self, url: &str) -> Result<&mut Self> {
        self.client = TransClient::new(Url::parse(url)?);
        Ok(self)
    }

    /// Updates [`Torrent`] values.
    pub async fn update(&mut self) -> &mut Self {
        self.torrents = self
            .client
            .torrent_get(self.fields.clone(), self.ids.clone())
            .await
            .unwrap()
            .arguments
            .torrents;
        self
    }

    /// Returns [`Vec`] of [`Torrent`] as reference.
    pub fn torrents(&self) -> &Vec<Torrent> {
        &self.torrents
    }
}

impl Debug for Torrents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fields: Vec<String> = match &self.fields {
            Some(fields) => fields.iter().map(|field| field.to_str()).collect(),
            None => vec![String::from("None")],
        };
        write!(
            f,
            "fields:
        {:?};\n\ntorrents: {:?}",
            fields, self.torrents
        )
    }
}
