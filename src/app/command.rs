use std::path::Path;

use anyhow::Result;
use transmission_rpc::types::{Id, Torrent, TorrentAction, TorrentStatus};

use super::Torrents;

impl Torrents {
    pub async fn toggle(&mut self, torrent: &Torrent) {
        let id = torrent.id().expect("ID not found");
        let action = match torrent.status {
            Some(TorrentStatus::Stopped) => TorrentAction::StartNow,
            _ => TorrentAction::Stop,
        };
        self.client
            .torrent_action(action, vec![id])
            .await
            .expect("Error toggling torrent");
    }

    pub async fn toggle_all(&mut self) {
        let torrents: Vec<(Id, TorrentAction)> = self
            .torrents
            .iter()
            .map(|torrent| {
                (
                    torrent.id().unwrap(),
                    match torrent.status {
                        Some(TorrentStatus::Stopped) => TorrentAction::StartNow,
                        _ => TorrentAction::Stop,
                    },
                )
            })
            .collect();

        for (id, action) in torrents {
            self.client
                .torrent_action(action, vec![id])
                .await
                .expect("Error toggling torrent");
        }
    }

    pub async fn start_all(&mut self) {
        self.action_all(TorrentAction::StartNow)
            .await
            .expect("Error starting all torrents");
    }

    pub async fn stop_all(&mut self) {
        self.action_all(TorrentAction::Stop)
            .await
            .expect("Error stopping all torrents");
    }

    pub async fn move_dir(
        &mut self,
        torrent: &Torrent,
        location: &Path,
        move_from: Option<bool>,
    ) -> transmission_rpc::types::Result<()> {
        let id = torrent.id().expect("ID not found");
        self.client
            .torrent_set_location(vec![id], location.to_string_lossy().into(), move_from)
            .await?;
        Ok(())
    }

    pub async fn delete(
        &mut self,
        torrent: &Torrent,
        delete_local_data: bool,
    ) -> transmission_rpc::types::Result<()> {
        let id = torrent.id().expect("ID not found");
        self.client
            .torrent_remove(vec![id], delete_local_data)
            .await?;
        Ok(())
    }

    pub fn rename(&mut self) -> Result<()> {
        todo!()
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
