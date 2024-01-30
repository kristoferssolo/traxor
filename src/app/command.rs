use anyhow::Result;
use transmission_rpc::types::{Id, Torrent, TorrentAction, TorrentStatus};

use super::Torrents;

impl Torrents {
    pub async fn toggle(&mut self, torrent: &Torrent) {
        let id = torrent.id().expect("ID not found");
        let action = match torrent.status.expect("Torrent status not found") {
            TorrentStatus::Stopped => TorrentAction::StartNow,
            _ => TorrentAction::Stop,
        };
        self.client
            .torrent_action(action, vec![id])
            .await
            .expect("Error toggling torrent");
    }

    pub async fn toggle_all(&mut self) {
        let torrents: Vec<(Id, TorrentAction)> = self
            .torrents()
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

    pub fn move_dir(&mut self) -> Result<()> {
        todo!()
    }

    pub fn delete(&mut self) -> Result<()> {
        todo!()
    }

    pub fn rename(&mut self) -> Result<()> {
        todo!()
    }
}
