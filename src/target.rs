use std::path::Path;

use crate::{AudioInfo, audio::{AudioError, AudioLocation, PlaylistName}, device::AttachedDevice};

// TRAIT: AudioTarget, e.g. an attached drive, the local file cache etc.
// AudioTarget impls are able to be written to, and can be used as a target for exporting audio from an AudioSource:
// 1. contains -> Look for existing AudioInfo in the target.
// 2. 
pub trait AudioTarget {
    fn name(&self) -> &str;
    fn contains(&self, info: &AudioInfo) -> Result<&AudioLocation, AudioError>;
    fn import(&self, source_path: &Path, info: &AudioInfo, playlist: Option<&PlaylistName>) -> Result<AudioLocation, AudioError>;
}

impl AudioTarget for AttachedDevice {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn contains(&self, info: &AudioInfo) -> Result<&AudioLocation, AudioError> {
        self.search(info)
    }
    
    fn import(&self, source_path: &Path, info: &AudioInfo, playlist: Option<&PlaylistName>) -> Result<AudioLocation, AudioError> {
        let dirpath = match &playlist.unwrap_or(&PlaylistName::Uncategorized) {
            PlaylistName::Uncategorized => self.path.clone(),
            PlaylistName::Named(name) => self.path.join(name),
        };
        
        // File name at destination will be same as source.
        let filename = source_path.file_name().unwrap().to_string_lossy().to_string();
        let dest_path = dirpath.join(filename);
        match std::fs::copy(&source_path, &dest_path) {
            Ok(num_bytes) => {
                println!("Copied {} bytes from {} to {}", num_bytes, source_path.display().to_string(), dest_path.display().to_string());
                Ok(AudioLocation::LocalPath(dest_path))
            },
            Err(e) => Err(AudioError::Io(e))
        }
    }
}