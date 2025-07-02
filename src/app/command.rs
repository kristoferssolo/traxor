use super::{types::Selected, Torrents};
use std::{collections::HashSet, path::Path};
use transmission_rpc::types::{Torrent, TorrentAction, TorrentStatus};

impl Torrents {
    pub async fn toggle(&mut self, ids: Selected) -> anyhow::Result<()> {
        let ids: HashSet<_> = ids.into();
        let torrents_to_toggle: Vec<_> = self
            .torrents
            .iter()
            .filter(|torrent| torrent.id.map_or(false, |id| ids.contains(&id)))
            .collect();

        for torrent in torrents_to_toggle {
            let action = match torrent.status {
                Some(TorrentStatus::Stopped) => TorrentAction::Start,
                _ => TorrentAction::Stop,
            };
            if let Some(id) = torrent.id() {
                self.client
                    .torrent_action(action, vec![id])
                    .await
                    .map_err(|e| anyhow::anyhow!("Transmission RPC error: {}", e.to_string()))?;
            }
        }
        Ok(())
    }

    pub async fn toggle_all(&mut self) -> anyhow::Result<()> {
        let torrents_to_toggle: Vec<_> = self
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

        for (id, action) in torrents_to_toggle {
            self.client
                .torrent_action(action, vec![id])
                .await
                .map_err(|e| anyhow::anyhow!("Transmission RPC error: {}", e.to_string()))?;
        }
        Ok(())
    }

    pub async fn start_all(&mut self) -> anyhow::Result<()> {
        self.action_all(TorrentAction::StartNow).await
    }

    pub async fn stop_all(&mut self) -> anyhow::Result<()> {
        self.action_all(TorrentAction::Stop).await
    }

    pub async fn move_dir(
        &mut self,
        torrent: &Torrent,
        location: &Path,
        move_from: Option<bool>,
    ) -> anyhow::Result<()> {
        if let Some(id) = torrent.id() {
            self.client
                .torrent_set_location(vec![id], location.to_string_lossy().into(), move_from)
                .await
                .map_err(|e| anyhow::anyhow!("Transmission RPC error: {}", e.to_string()))?;
        }
        Ok(())
    }

    pub async fn delete(&mut self, ids: Selected, delete_local_data: bool) -> anyhow::Result<()> {
        self.client
            .torrent_remove(ids.into(), delete_local_data)
            .await
            .map_err(|e| anyhow::anyhow!("Transmission RPC error: {}", e.to_string()))?;
        Ok(())
    }

    pub async fn rename(&mut self, torrent: &Torrent, name: &Path) -> anyhow::Result<()> {
        if let (Some(id), Some(old_name)) = (torrent.id(), torrent.name.clone()) {
            self.client
                .torrent_rename_path(vec![id], old_name, name.to_string_lossy().into())
                .await
                .map_err(|e| anyhow::anyhow!("Transmission RPC error: {}", e.to_string()))?;
        }
        Ok(())
    }

    async fn action_all(&mut self, action: TorrentAction) -> anyhow::Result<()> {
        let ids = self
            .torrents
            .iter()
            .filter_map(|torrent| torrent.id())
            .collect::<Vec<_>>();

        self.client
            .torrent_action(action, ids)
            .await
            .map_err(|e| anyhow::anyhow!("Transmission RPC error: {}", e.to_string()))?;
        Ok(())
    }
}
