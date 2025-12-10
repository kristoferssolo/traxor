use super::{Torrents, types::Selected};
use crate::error::Result;
use std::{collections::HashSet, path::Path};
use transmission_rpc::types::{Torrent, TorrentAction, TorrentStatus};

impl Torrents {
    /// # Errors
    ///
    /// TODO: add error types
    pub async fn toggle(&mut self, ids: Selected) -> Result<()> {
        let ids: HashSet<_> = ids.into();
        let torrents_to_toggle = self
            .torrents
            .iter()
            .filter(|torrent| torrent.id.is_some_and(|id| ids.contains(&id)))
            .collect::<Vec<_>>();

        for torrent in torrents_to_toggle {
            let action = match torrent.status {
                Some(TorrentStatus::Stopped) => TorrentAction::Start,
                _ => TorrentAction::Stop,
            };
            if let Some(id) = torrent.id() {
                self.client.torrent_action(action, vec![id]).await?;
            }
        }
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn toggle_all(&mut self) -> Result<()> {
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
            self.client.torrent_action(action, vec![id]).await?;
        }
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn start_all(&mut self) -> Result<()> {
        self.action_all(TorrentAction::StartNow).await
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn stop_all(&mut self) -> Result<()> {
        self.action_all(TorrentAction::Stop).await
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn move_dir(
        &mut self,
        torrent: &Torrent,
        location: &Path,
        move_from: Option<bool>,
    ) -> Result<()> {
        if let Some(id) = torrent.id() {
            self.client
                .torrent_set_location(vec![id], location.to_string_lossy().into(), move_from)
                .await?;
        }
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn delete(&mut self, ids: Selected, delete_local_data: bool) -> Result<()> {
        self.client
            .torrent_remove(ids.into(), delete_local_data)
            .await?;
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn rename(&mut self, torrent: &Torrent, name: &Path) -> Result<()> {
        if let (Some(id), Some(old_name)) = (torrent.id(), torrent.name.clone()) {
            self.client
                .torrent_rename_path(vec![id], old_name, name.to_string_lossy().into())
                .await?;
        }
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: add error types
    async fn action_all(&mut self, action: TorrentAction) -> Result<()> {
        let ids = self
            .torrents
            .iter()
            .filter_map(Torrent::id)
            .collect::<Vec<_>>();

        self.client.torrent_action(action, ids).await?;
        Ok(())
    }
}
