use super::{Torrents, types::Selected};
use crate::error::Result;
use std::{collections::HashSet, path::Path};
use transmission_rpc::types::{Id, Torrent, TorrentAction, TorrentStatus};

impl Torrents {
    /// Toggle selected torrents between started and stopped states.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn toggle(&mut self, ids: Selected) -> Result<()> {
        let selected: HashSet<_> = ids.into();

        let (to_start, to_stop): (Vec<_>, Vec<_>) = self
            .torrents
            .iter()
            .filter_map(|t| {
                t.id.filter(|id| selected.contains(id))
                    .map(|id| (Id::Id(id), t.status))
            })
            .partition(|(_, status)| matches!(status, Some(TorrentStatus::Stopped)));

        if !to_start.is_empty() {
            let ids: Vec<_> = to_start.into_iter().map(|(id, _)| id).collect();
            self.client.torrent_action(TorrentAction::Start, ids).await?;
        }
        if !to_stop.is_empty() {
            let ids: Vec<_> = to_stop.into_iter().map(|(id, _)| id).collect();
            self.client.torrent_action(TorrentAction::Stop, ids).await?;
        }
        Ok(())
    }

    /// Toggle all torrents between started and stopped states.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn toggle_all(&mut self) -> Result<()> {
        let (to_start, to_stop): (Vec<_>, Vec<_>) = self
            .torrents
            .iter()
            .filter_map(|t| t.id().map(|id| (id, t.status)))
            .partition(|(_, status)| matches!(status, Some(TorrentStatus::Stopped)));

        if !to_start.is_empty() {
            let ids: Vec<_> = to_start.into_iter().map(|(id, _)| id).collect();
            self.client
                .torrent_action(TorrentAction::StartNow, ids)
                .await?;
        }
        if !to_stop.is_empty() {
            let ids: Vec<_> = to_stop.into_iter().map(|(id, _)| id).collect();
            self.client.torrent_action(TorrentAction::Stop, ids).await?;
        }
        Ok(())
    }

    /// Start all torrents immediately.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn start_all(&mut self) -> Result<()> {
        self.action_all(TorrentAction::StartNow).await
    }

    /// Stop all torrents.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn stop_all(&mut self) -> Result<()> {
        self.action_all(TorrentAction::Stop).await
    }

    /// Move a torrent to a new location.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn move_dir(
        &mut self,
        torrent: &Torrent,
        location: &Path,
        move_from: Option<bool>,
    ) -> Result<()> {
        let Some(id) = torrent.id() else {
            return Ok(());
        };
        self.client
            .torrent_set_location(vec![id], location.to_string_lossy().into_owned(), move_from)
            .await?;
        Ok(())
    }

    /// Delete torrents, optionally removing local data.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn delete(&mut self, ids: Selected, delete_local_data: bool) -> Result<()> {
        self.client
            .torrent_remove(ids.into(), delete_local_data)
            .await?;
        Ok(())
    }

    /// Move torrents to a new location.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn move_torrents(&mut self, ids: Selected, location: &str) -> Result<()> {
        let ids: Vec<Id> = ids.into();
        if ids.is_empty() {
            return Ok(());
        }
        self.client
            .torrent_set_location(ids, location.to_owned(), Some(true))
            .await?;
        Ok(())
    }

    /// Rename a torrent.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn rename(&mut self, torrent: &Torrent, name: &Path) -> Result<()> {
        let (Some(id), Some(old_name)) = (torrent.id(), torrent.name.as_ref()) else {
            return Ok(());
        };
        self.client
            .torrent_rename_path(vec![id], old_name.clone(), name.to_string_lossy().into_owned())
            .await?;
        Ok(())
    }

    async fn action_all(&mut self, action: TorrentAction) -> Result<()> {
        let ids: Vec<_> = self.torrents.iter().filter_map(Torrent::id).collect();
        if !ids.is_empty() {
            self.client.torrent_action(action, ids).await?;
        }
        Ok(())
    }
}
