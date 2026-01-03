// Cache represents the local file cache for music-man, its an intermediate between AudioSource and AudioTarget e.g.
// sources when fetching data will write the local file cache, while targets will import from it, rather than any direct
// interop between sources and targets.
//
// We use a flat device structure along with a metadata file for maintaining playlist and other metastate on top of the local cache
// e.g. if we want to sync a playlist from spotify to some attached device, we would fetch all the audio in flat form to the local cache
// and maintain a durable playlist entry.
//
// Cache is an AudioIndex and an AudioSource

use std::{collections::HashMap, path::Path};
use std::fs::{create_dir_all, read_dir, read_to_string, write};
use std::path::PathBuf;

use crate::audio::list_audio_in_folder;
use crate::source::AudioSource;
use crate::{audio::{AudioError, AudioInfo, AudioKey, AudioLocation, Playlist, PlaylistName}, index::AudioIndex};

pub fn setup_app_directories() -> std::io::Result<()> {
    let data_dir = get_data_dir();
    let config_dir = get_config_dir();
    let cache_dir = get_cache_dir();

    // Create base app directories.
    create_dir_all(&data_dir)?;
    create_dir_all(&config_dir)?;
    create_dir_all(&cache_dir)?;

    // Create local file cache for downloaded audio.
    create_dir_all(audio_cache_dir())?;

    println!("Data dir: {:?}", data_dir);
    println!("Cache dir: {:?}", cache_dir);
    println!("Config dir: {:?}", config_dir);
    Ok(())
}

pub fn get_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().unwrap().join("music-man")
    }
}

pub fn get_config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::config_dir().unwrap().join("music-man")
    }
}

pub fn get_cache_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::cache_dir().unwrap().join("music-man")
    }
}

pub fn audio_cache_dir() -> PathBuf {
    get_cache_dir().join("audio")
}

pub fn playlist_cache() -> PathBuf {
    audio_cache_dir().join("playlists.json")
}

#[derive(Clone, Debug)]
pub struct LocalCache {
    // flat cache directory for all audio.
    audio_dir: PathBuf,
    // Maps audio to cache locations.
    index: HashMap<AudioKey, PathBuf>,
    // Overlays the flat cache with playlist mappings.
    playlists: HashMap<String, Vec<AudioInfo>>,
    // Path to saved playlists metadata.
    playlists_path: PathBuf,
}

impl LocalCache {
    pub fn new() -> Self {
        // Load system state from config + data + cache directories.
        // 1. local file cache, for existing audio.
        // 2. audio lookup map -> mapping (artist, song) -> audio file.
        // 3. playlist map -> mapping (playlist name) -> set of AudioInfo.
        setup_app_directories().expect("Failed to create app directories.");
        let audio_dir = audio_cache_dir();
        let playlists_path = playlist_cache();
        let mut cache = Self {
            audio_dir,
            index: HashMap::new(),
            playlists: Self::load_playlists(&playlists_path),
            playlists_path,
        };
        cache.rebuild_index();
        println!("Initialized Local Cache: {:?}", cache);
        cache
    }

    pub fn search_playlist(&self, playlist_name: &str) -> Result<Vec<(&AudioInfo, AudioLocation)>, AudioError> {
        let playlist = self.get_playlist(playlist_name).ok_or(AudioError::NotFound)?;
        playlist
            .iter()
            .map(|info| {
                let location = self.search(info)?;
                Ok((info, location))
            })
            .collect()
    }

    pub fn list_playlist_names(&self) -> impl Iterator<Item = &str> {
        self.playlists.keys().map(|s| s.as_str())
    }

    pub fn search(&self, info: &AudioInfo) -> Result<AudioLocation, AudioError> {
        let path = self.search_path(info)?;
        Ok(AudioLocation::LocalPath(path.to_path_buf()))
    }

    fn search_path(&self, info: &AudioInfo) -> Result<&PathBuf, AudioError> {
        let key = AudioKey::from_info(info).ok_or(AudioError::MissingInfo)?;
        self.index.get(&key)
            .ok_or(AudioError::NotFound)
    }

    /// Add downloaded audio to the cache index, and optionally to a playlist.
    /// Call this after fetching audio from a source.
    pub fn add_to_cache(&mut self, info: &AudioInfo, location: &AudioLocation, playlist: Option<&str>) {
        // Update the index
        if let AudioLocation::LocalPath(path) = location {
            if let Some(key) = AudioKey::from_info(info) {
                self.index.insert(key, path.clone());
            }
        }

        // Add to playlist if specified
        if let Some(playlist_name) = playlist {
            self.add_to_playlist(playlist_name, info.clone());
        }
    }

    // Iterate the disk cache and build the index of AudioKey -> Audio path.
    fn rebuild_index(&mut self) {
        self.index.clear();

        if let Ok(entries) = read_dir(&self.audio_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.path().is_file() {
                    let filename = entry.file_name().to_string_lossy().to_string();
                    if filename.starts_with("._") {
                        continue;
                    }
                    
                    let info = AudioInfo::from_filename(&filename);
                    if let Some(key) = AudioKey::from_info(&info) {
                        self.index.insert(key, entry.path());
                    }
                }
            }
        }
    }

    // Reload the on-disk playlists file.
    fn load_playlists(path: &Path) -> HashMap<String, Vec<AudioInfo>> {
        read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_playlists(&self) -> std::io::Result<()> {
        let playlist_json = serde_json::to_string_pretty(&self.playlists)?;
        write(&self.playlists_path, playlist_json)
    }

    fn get_playlist(&self, name: &str) -> Option<&Vec<AudioInfo>> {
        self.playlists.get(name)
    }

    // Add audio to a playlist and save the playlist file.
    fn add_to_playlist(&mut self, playlist_name: &str, audio: AudioInfo) {
        self.playlists
            .entry(playlist_name.to_string())
            .or_default()
            .push(audio);
        self.save_playlists().ok();
    }
}

impl AudioIndex for LocalCache {
    fn name(&self) -> &str {
        "Local Cache"
    }

    fn list_playlists(&self) -> Result<Vec<Playlist>, AudioError> {
        // Build all of the "real" playlists (not uncategorized audio).
        let mut result: Vec<Playlist> = self.playlists
            .iter()
            .map(|(name, tracks)| Playlist {
                name: PlaylistName::Named(name.clone()),
                audio: tracks.clone(),
            })
            .collect();
        
        // Also list all cached files as "Uncategorized"
        let all_cached = list_audio_in_folder(&self.audio_dir)?;
        
        if !all_cached.is_empty() {
            result.push(Playlist {
                name: PlaylistName::Uncategorized,
                audio: all_cached,
            });
        }
        
        Ok(result)
    }
}

impl AudioSource for LocalCache {
    fn name(&self) -> &str {
        "Local Cache"
    }

    fn search(&self, query: &AudioInfo) -> Result<AudioInfo, AudioError> {
        // Check if we have this in the index
        let _ = self.search(query)?;
        Ok(query.clone())
    }
    
    // LocalCache is unique vs. any other AudioSource in that its just the cache that buffers
    // audio between sources and targets. Nominally it is a "source" and we should be able to fetch
    // on it, but in reality this will just be used to get back the local cache location of the audio.
    // Exception would be if we are trying to fetch to the cache with some AudioInfo that matches a
    // cached path, but the destination path we fetch to is different.
    fn fetch(&self, info: &AudioInfo, dest: PathBuf) -> Result<AudioLocation, AudioError> {
        // Already in cache - just return the path
        let cached_path = self.search_path(info)?;
        
        // If dest is different from cache dir, copy the file
        if dest != self.audio_dir {
            let filename = cached_path.file_name().ok_or(AudioError::NotFound)?;
            let dest_path = dest.join(filename);
            std::fs::copy(cached_path, &dest_path)?;
            Ok(AudioLocation::LocalPath(dest_path))
        } else {
            Ok(AudioLocation::LocalPath(cached_path.clone()))
        }
    }
}