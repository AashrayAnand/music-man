use std::fmt::Display;
use std::path::PathBuf;
use std::fs::DirEntry;

use crate::source::AudioSource;
use crate::target::AudioTarget;

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

// AudioInfo -> A structure representing various information about audio. Depending on the information present, it can
// be used for searching different AudioSource and AudioTarget, to see where the audio resides already.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AudioInfo {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub filename: Option<String>,
    pub youtube_url: Option<String>,
    pub isrc: Option<String>,
    pub duration_secs: Option<u32>
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
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

struct AudioLookup {
    info: AudioInfo,
    location: AudioLocation,
}

// 1. Check if the AudioInfo exists on the source.
//pub fn transfer<S: AudioSource, T: AudioTarget>(source: S, target: T, info: &AudioInfo) -> Result<AudioLocation, AudioError> {
//    let source_info = source.search(info)?;
//    let intermediate_transfer = source.fetch(&source_info, dest)
//}

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