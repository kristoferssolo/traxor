use std::path::Path;

use tracing::debug;
use transmission_rpc::types::{Id, Torrent, TorrentAction, TorrentStatus};

use super::Torrents;

impl Torrents {
    pub async fn toggle(&mut self, ids: &Vec<i64>) {
        debug!("ID list - {:?}", ids);

        let api_ids: Vec<_> = self
            .torrents
            .iter()
            .filter_map(|torrent| {
                if let Some(id) = torrent.id {
                    if ids.contains(&id) {
                        return torrent.id();
                    }
                }
                return None;
            })
            .collect();

        debug!("API ID list - {:?}", api_ids);

        for torrent in &self.torrents {
            if let Some(id) = torrent.id {
                if ids.contains(&id) {
                    let action = match torrent.status {
                        Some(TorrentStatus::Stopped) => TorrentAction::Start,
                        _ => TorrentAction::Stop,
                    };
                    if !api_ids.is_empty() {
                        if let Err(err) = self.client.torrent_action(action, api_ids.clone()).await
                        {
                            eprintln!("Error toggling torrent: {}", err);
                        }
                    }
                }
            }
        }
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

    pub async fn delete(&mut self, ids: &Vec<i64>, delete_local_data: bool) {
        let api_ids = self
            .torrents
            .iter()
            .filter(|torrent| ids.contains(&torrent.id.unwrap()))
            .map(|torrent| torrent.id().unwrap())
            .collect();

        self.client
            .torrent_remove(api_ids, delete_local_data)
            .await
            .unwrap();
    }

    pub async fn rename(
        &mut self,
        torrent: &Torrent,
        name: &Path,
    ) -> transmission_rpc::types::Result<()> {
        let id = torrent.id().expect("ID not found");
        let old_name = torrent.name.clone().unwrap();
        self.client
            .torrent_rename_path(vec![id], old_name, name.to_string_lossy().into())
            .await?;
        Ok(())
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
