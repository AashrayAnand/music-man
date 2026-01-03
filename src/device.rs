use crate::{
    audio::{AudioError, AudioInfo, AudioKey, AudioLocation, PlaylistName},
    index::AudioIndex,
};
use std::{
    collections::HashMap,
    path::PathBuf,
};

// An attached device e.g. mp3 player, hard drive etc,
#[derive(Clone, Debug)]
pub struct AttachedDevice {
    pub name: String,
    pub path: PathBuf,
    index: HashMap<AudioKey, AudioLocation>,
}

impl AttachedDevice {
    pub fn new(name: String, path: PathBuf) -> Self {
        let mut device = Self {
            name,
            path,
            index: HashMap::new(),
        };
        // Iterate the device to construct a local index.
        let playlists = device.list_playlists().unwrap();
        for playlist in &playlists {
            // We treat all files stored in the root of an AttachedDevice as being part of a special ""
            let dirpath = match &playlist.name {
                PlaylistName::Uncategorized => device.path.clone(),
                PlaylistName::Named(name) => device.path.join(name),
            };

            for audio in &playlist.audio {
                if let Some(audiokey) = AudioKey::from_info(audio) {
                    let audiopath: PathBuf = dirpath.join(
                        audio
                            .filename
                            .as_ref()
                            .expect("AttachedDevice must have audio filenames."),
                    );
                    device
                        .index
                        .insert(audiokey, AudioLocation::LocalPath(audiopath));
                }
            }
        }
        println!("Added new attached device: {:?}", device);

        device
    }

    pub fn search(&self, info: &AudioInfo) -> Result<&AudioLocation, AudioError> {
        let key = AudioKey::from_info(info).ok_or(AudioError::MissingInfo)?;
        self.index.get(&key).ok_or(AudioError::NotFound)
    }

    pub fn update_index(
        &mut self,
        info: &AudioInfo,
        location: &AudioLocation,
    ) -> Result<(), AudioError> {
        if let AudioLocation::LocalPath(_) = location {
            if let Some(audiokey) = AudioKey::from_info(info) {
                self.index.insert(audiokey, location.clone());
                return Ok(());
            }
        }
        Err(AudioError::Unexpected)
    }

}
