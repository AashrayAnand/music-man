use std::{fs::DirEntry, path::{Path, PathBuf}};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaylistName {
    Named(String),
    Uncategorized,
}

impl PlaylistName {
    pub fn disp_name(&self) -> &str {
        match self {
            PlaylistName::Named(s) => s,
            PlaylistName::Uncategorized => "Uncategorized",
        }
    }
}

// A collection of AudioInfo.
pub struct Playlist {
    pub name: PlaylistName,
    pub audio: Vec<AudioInfo>,
}

// A hashable key for indexing audio by artist + title.
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct AudioKey {
    pub artist: String,
    pub title: String,
}

impl AudioKey {
    pub fn from_info(info: &AudioInfo) -> Option<Self> {
        Some(Self {
            artist: info.artist.as_ref()?.to_lowercase(),
            title: info.title.as_ref()?.to_lowercase(),
        })
    }
}

// AudioInfo -> A structure representing various information about audio. Depending on the information present, it can
// be used for searching different AudioSource and AudioTarget, to see where the audio resides already.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AudioInfo {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub filename: Option<String>,
    pub youtube_url: Option<String>,
    pub isrc: Option<String>,
    pub duration_secs: Option<u32>,
}

impl AudioInfo {
    pub fn from_filename(filename: impl AsRef<Path>) -> Self {
        let filename_str = filename.as_ref().to_string_lossy();
        let stem = filename
            .as_ref()
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or(filename_str.clone());

        // Try to split up the filename to artist + title, if delimiter isn't there just take it all as title.
        let (artist, title) = stem
            .split_once(" - ")
            .or_else(|| stem.split_once(" – ")) // en-dash
            .map(|(a, t)| (Some(a.trim().to_string()), Some(t.trim().to_string())))
            .unwrap_or((None, Some(stem.to_string())));

        Self {
            artist,
            title,
            filename: Some(filename_str.to_string()), // AttachedDevice will always have at least filenames.
            youtube_url: None,
            isrc: None,
            duration_secs: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Unexpected Behavior")]
    Unexpected,
    #[error("Audio not found")]
    NotFound,
    #[error("Missing Audio Info")]
    MissingInfo,
    #[error("Source unavailable: {0}")]
    Unavailable(String),
    #[error("Export failed: {0}")]
    ExportFailed(String),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}

// Represents an audio location, with varying types for different location implementations.
#[derive(Clone, Debug)]
pub enum AudioLocation {
    LocalPath(PathBuf),
    RemoteUrl(String),
}

impl AudioLocation {
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self::LocalPath(path.into())
    }

    pub fn remote(url: impl Into<String>) -> Self {
        Self::RemoteUrl(url.into())
    }
}

pub fn is_supported_audio_file(entry: &DirEntry) -> bool {
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

/// List all supported audio files in a folder, returning AudioInfo for each.
pub fn list_audio_in_folder(folder: &Path) -> Result<Vec<AudioInfo>, AudioError> {
    std::fs::read_dir(folder)?
        .filter_map(|e| e.ok())
        .filter(|entry| is_supported_audio_file(entry))
        .map(|entry| Ok(AudioInfo::from_filename(&entry.file_name())))
        .collect()
}