use std::{collections::HashMap, fs::DirEntry, hash::Hash, path::{Path, PathBuf}};
use crate::{audio::{AudioError, AudioInfo, AudioLocation, PlaylistName}, index::AudioIndex};

fn is_supported_audio_file(entry: &DirEntry) -> bool {
    if !entry.path().is_file() {
        return false;
    }

    // macOS fork files.
    if entry.file_name().to_string_lossy().starts_with("._") {
        return false;
    }

    entry
        .path()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "mp3" | "flac" | "wma" | "wav" | "aac" | "m4a" | "ape"
            )
        })
        .unwrap_or(false)
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
struct AudioKey {
    artist: String,
    title: String,
}

impl AudioKey {
    fn from_info(info: &AudioInfo) -> Option<Self> {
        // Will be None if AudioInfo doesn't provide artist or title.
        Some(Self { artist: info.artist.as_ref()?.to_lowercase(), title: info.title.as_ref()?.to_lowercase() })
    }
}

// An attached device e.g. mp3 player, hard drive etc, 
#[derive(Clone, Debug)]
pub struct AttachedDevice {
    pub name: String,
    pub path: PathBuf,
    index: HashMap<AudioKey, AudioLocation>,
}

impl AttachedDevice {
    pub fn new(name: String, path: PathBuf) -> Self {
        let mut device = Self { name, path, index: HashMap::new() };
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
                    let audiopath: PathBuf = dirpath.join(audio.filename.as_ref().expect("AttachedDevice must have audio filenames."));
                    device.index.insert(audiokey, AudioLocation::LocalPath(audiopath));
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

    pub fn list_audio_in_folder(&self, folder: &Path) -> Result<Vec<AudioInfo>, AudioError> {
        std::fs::read_dir(folder)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|entry| is_supported_audio_file(entry))
            .map(|entry| {
                let filename = entry.path();
                Ok(AudioInfo::from_filename(&filename))
            })
            .collect()
    }
}