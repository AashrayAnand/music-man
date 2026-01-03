use crate::{
    AudioInfo,
    audio::{AudioError, AudioLocation, PlaylistName},
    device::AttachedDevice,
};

// TRAIT: AudioTarget, e.g. an attached drive, the local file cache etc.
// AudioTarget impls are able to be written to, and can be used as a target for exporting audio from an AudioSource:
// 1. contains -> Look for existing AudioInfo in the target.
// 2. import -> Import audio to this target into a specified playist (if any), from a provided source location.
pub trait AudioTarget {
    fn name(&self) -> &str;
    fn contains(&self, info: &AudioInfo) -> Result<&AudioLocation, AudioError>;
    fn import(
        &self,
        source_location: &AudioLocation,
        info: &AudioInfo,
        playlist: Option<PlaylistName>,
    ) -> Result<AudioLocation, AudioError>;
}

impl AudioTarget for AttachedDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn contains(&self, info: &AudioInfo) -> Result<&AudioLocation, AudioError> {
        self.search(info)
    }

    // As of now we don't need AudioInfo to import to an AttachedDevice, for the currently support option of device local transfer,
    // we just re-use the existing location filename when copying to the target.
    fn import(
        &self,
        source_location: &AudioLocation,
        _info: &AudioInfo,
        playlist: Option<PlaylistName>,
    ) -> Result<AudioLocation, AudioError> {
        match source_location {
            AudioLocation::LocalPath(source_path) => {
                let dirpath = match &playlist.unwrap_or(PlaylistName::Uncategorized) {
                    PlaylistName::Uncategorized => self.path.clone(),
                    PlaylistName::Named(name) => self.path.join(name),
                };

                // Ensure the playlist directory exists.
                std::fs::create_dir_all(&dirpath)?;

                // File name at destination will be same as source.
                let filename = source_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                let dest_path = dirpath.join(filename);
                match std::fs::copy(&source_path, &dest_path) {
                    Ok(num_bytes) => {
                        println!(
                            "Copied {} bytes from {} to {}",
                            num_bytes,
                            source_path.display().to_string(),
                            dest_path.display().to_string()
                        );
                        Ok(AudioLocation::LocalPath(dest_path))
                    }
                    Err(e) => Err(AudioError::Io(e)),
                }
            }
            _ => Err(AudioError::ExportFailed(format!(
                "Currently do not support import to AttachedDevice from non-LocalPath."
            ))),
        }
    }
}