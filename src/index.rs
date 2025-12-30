use crate::audio::{AudioError, Playlist, PlaylistName};
use crate::device::AttachedDevice;

// TRAIT: AudioIndex, e.g. an attached mp3 device, a streaming platform, etc.
// AudioIndex impls are able to specify an index of AudioInfo. They may not necessarily be AudioSource or AudioTarget that we can read/write,
// but they at least provide an index of information about audio e.g. Spotify can provide an index of the user's spotify playlists and music.
// 1. list -> returns a corresponding AudioCollection describing the music on the device.
// 2. 
pub trait AudioIndex {
    fn name(&self) -> &str;
    fn list_playlists(&self) -> Result<Vec<Playlist>, AudioError>;
}

impl AudioIndex for AttachedDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn list_playlists(&self) -> Result<Vec<Playlist>, AudioError> {
        // Iterate device directories, list out all directories 
        let mut playlists = Vec::new();

        // Add a playlist entry per-directory, and an uncategorized playlist for all root files.
        let playlist_directories = std::fs::read_dir(&self.path)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.as_ref().unwrap();
                let file_name = entry.file_name().to_string_lossy().to_string();
                if entry.file_type().unwrap().is_dir() 
                    && !file_name.starts_with('.')
                    && file_name != "System Volume Information" {
                    return Some((entry.path(), file_name))
                }
                None
            });
        
        for (directory, dirname) in playlist_directories {
            let audio = self.list_audio_in_folder(&directory)?;
            playlists.push(Playlist { name: PlaylistName::Named(dirname), audio });
        }

        if let Ok(root_playlist) = self.list_audio_in_folder(&self.path) {
            if !root_playlist.is_empty() {
                playlists.push(Playlist { name: PlaylistName::Uncategorized, audio: root_playlist });
            }
        }

        Ok(playlists)
    }
}