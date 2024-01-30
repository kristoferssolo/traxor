use anyhow::Result;
use transmission_rpc::types::{Torrent, TorrentAction, TorrentStatus};

use super::Torrents;

impl Torrents {
    pub async fn toggle(&mut self, torrent: &Torrent) -> transmission_rpc::types::Result<()> {
        let id = torrent.id().ok_or_else(|| "ID not found")?;
        let action = match torrent.status.ok_or_else(|| "Torrent status not found")? {
            TorrentStatus::Stopped => TorrentAction::StartNow,
            _ => TorrentAction::Stop,
        };
        self.client.torrent_action(action, vec![id]).await?;
        Ok(())
    }

    pub async fn toggle_all(&mut self) -> transmission_rpc::types::Result<()> {
        todo!()
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
