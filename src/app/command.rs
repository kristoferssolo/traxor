use std::{collections::HashSet, path::Path};

use tracing::error;
use transmission_rpc::types::{Torrent, TorrentAction, TorrentStatus};

use super::{types::Selected, Torrents};

impl Torrents {
    pub async fn toggle(&mut self, ids: Selected) {
        let ids: HashSet<_> = ids.into();
        let torrents = self.torrents.iter().filter(|torrent| {
            if let Some(id) = torrent.id {
                return ids.contains(&id);
            }
            false
        });

        for torrent in torrents {
            let action = match torrent.status {
                Some(TorrentStatus::Stopped) => TorrentAction::Start,
                _ => TorrentAction::Stop,
            };
            if let Some(id) = torrent.id() {
                if let Err(e) = self.client.torrent_action(action, vec![id]).await {
                    error!("{:?}", e);
                }
            }
        }
    }

    pub async fn toggle_all(&mut self) {
        let torrents: Vec<_> = self
            .torrents
            .iter()
            .filter_map(|torrent| {
                torrent.id().map(|id| {
                    (
                        id,
                        match torrent.status {
                            Some(TorrentStatus::Stopped) => TorrentAction::StartNow,
                            _ => TorrentAction::Stop,
                        },
                    )
                })
            })
            .collect();

        for (id, action) in torrents {
            if let Err(e) = self.client.torrent_action(action, vec![id]).await {
                error!("{:?}", e);
            }
        }
    }

    pub async fn start_all(&mut self) {
        if let Err(e) = self.action_all(TorrentAction::StartNow).await {
            error!("{:?}", e);
        }
    }

    pub async fn stop_all(&mut self) {
        if let Err(e) = self.action_all(TorrentAction::Stop).await {
            error!("{:?}", e);
        }
    }

    pub async fn move_dir(&mut self, torrent: &Torrent, location: &Path, move_from: Option<bool>) {
        if let Some(id) = torrent.id() {
            if let Err(e) = self
                .client
                .torrent_set_location(vec![id], location.to_string_lossy().into(), move_from)
                .await
            {
                error!("{:?}", e);
            }
        }
    }

    pub async fn delete(&mut self, ids: Selected, delete_local_data: bool) {
        if let Err(e) = self
            .client
            .torrent_remove(ids.into(), delete_local_data)
            .await
        {
            error!("{:?}", e);
        }
    }

    pub async fn rename(&mut self, torrent: &Torrent, name: &Path) {
        if let (Some(id), Some(old_name)) = (torrent.id(), torrent.name.clone()) {
            if let Err(e) = self
                .client
                .torrent_rename_path(vec![id], old_name, name.to_string_lossy().into())
                .await
            {
                error!("{:?}", e);
            }
        }
    }

    async fn action_all(&mut self, action: TorrentAction) -> transmission_rpc::types::Result<()> {
        let ids = self
            .torrents
            .iter()
            .filter_map(|torrent| torrent.id())
            .collect::<Vec<_>>();

        self.client.torrent_action(action, ids).await?;
        Ok(())
    }
}
