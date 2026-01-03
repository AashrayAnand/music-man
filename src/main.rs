pub mod cache;
pub mod audio;
pub mod device;
pub mod index;
pub mod source;
pub mod target;

use std::{io::stdin, path::PathBuf};

use crate::{
    cache::{get_cache_dir, audio_cache_dir, setup_app_directories, LocalCache},
    audio::{AudioError, AudioInfo, PlaylistName},
    device::AttachedDevice,
    index::AudioIndex,
    source::{AudioSource, YtDlpSource},
    target::AudioTarget,
};

fn main() {
    // 1. Get user device to download audio to.
    let dirpath = {
        println!(
            "Provide a directory (empty -> {})",
            audio_cache_dir().to_string_lossy()
        );
        let mut directory = String::new();
        stdin()
            .read_line(&mut directory)
            .expect("Failed to read line");

        if directory.trim().is_empty() {
            get_cache_dir().join("audio")
        } else {
            PathBuf::from(&directory.trim())
        }
    };

    let source = YtDlpSource {
        name: "ytdlp".to_string(),
    };
    let mut cache = LocalCache::new();
    let target = AttachedDevice::new(dirpath.display().to_string(), dirpath);

    // Iterate sources in order, until we find one that contains the AudioInfo.
    // Fetch from the source to the local file cache, will mean we cache the audio there for a future look up.
    loop {
        print!("> ");
        let mut buffer = String::new();
        stdin()
            .read_line(&mut buffer)
            .expect("Failed to read input");

        let mut split = buffer.trim().split_whitespace();
        let cmd = split.next().expect("No command provided.");
        let args = split.collect::<Vec<_>>();
        match cmd {
            "list" => {
                for playlist in target.list_playlists().unwrap() {
                    println!("Playlist {}", playlist.name.disp_name());
                    for audio in &playlist.audio {
                        println!("{:?}", audio);
                    }
                }
            }
            "search" => {
                let artist = args.get(0).expect("Usage: search '<artist>' '<title>'");
                let title = args.get(1).expect("Usage: search '<artist>' '<title>'");
                let info = AudioInfo {
                    artist: Some(artist.to_string()),
                    title: Some(title.to_string()),
                    ..Default::default()
                };

                if let Ok(loc) = cache.search(&info) {
                    println!(
                        "Found {}: {} in the local file cache at {:?}.",
                        artist, title, loc
                    );
                }
            }
            "download" => {
                // Parse: download <url> [playlist] OR download <artist> <title> [playlist]
                let is_youtube_url = args.get(0).map(|a| a.to_lowercase().contains("youtube")).unwrap_or(false);
                
                let (info, playlist) = if is_youtube_url {
                    // download <url> [playlist]
                    let info = AudioInfo {
                        youtube_url: Some(args[0].to_string()),
                        ..Default::default()
                    };
                    let playlist = args.get(1).map(|s| s.to_string());
                    (info, playlist)
                } else {
                    // download <artist> <title> [playlist]
                    let info = AudioInfo {
                        artist: Some(args.get(0).expect("Usage: download <url> [playlist] OR download <artist> <title> [playlist]").to_string()),
                        title: Some(args.get(1).expect("Usage: download <url> [playlist] OR download <artist> <title> [playlist]").to_string()),
                        ..Default::default()
                    };
                    let playlist = args.get(2).map(|s| s.to_string());
                    (info, playlist)
                };

                match source.fetch(&info, audio_cache_dir()) {
                    Ok(location) => {
                        cache.add_to_cache(&info, &location, playlist.as_deref());
                        println!("Downloaded to cache: {:?}", location);
                        if let Some(p) = &playlist {
                            println!("Added to playlist: {}", p);
                        }
                    },
                    Err(e) => println!("Download failed: {:?}", e),
                }
            }
            "import" => {
                let artist = args
                    .get(0)
                    .expect("Usage: import <artist> <title> [playlist]");
                let title = args
                    .get(1)
                    .expect("Usage: import <artist> <title> [playlist]");

                // If playlist is provided, import to it, otherwise will assume the audio can be uncategorized.
                let playlist = args.get(2).map(|s| PlaylistName::Named(s.to_string()));
                let info = AudioInfo {
                    artist: Some(artist.to_string()),
                    title: Some(title.to_string()),
                    ..Default::default()
                };

                let location = cache.search(&info);
                match location {
                    Ok(_) => match target.import(&location.unwrap(), &info, playlist) {
                        Ok(loc) => println!("Imported to target: {:?}", loc),
                        Err(e) => println!("Import failed {:?}", e),
                    },
                    Err(e) => match e {
                        AudioError::MissingInfo => println!(
                            "Failed to find {:?} in cache, missing some required info.",
                            info
                        ),
                        AudioError::NotFound => println!(
                            "Failed to find {:?} in cache, need to run 'download' first.",
                            info
                        ),
                        _ => panic!("Unexpected error during import."),
                    },
                }
            }
            "list_playlists" => {
                for name in cache.list_playlist_names() {
                    println!("{}", name);
                }
            }
            "show_playlist" => {
                let playlist_name = args
                    .get(0)
                    .expect("Usage: show_playlist <playlist>");
                match cache.search_playlist(playlist_name) {
                    Ok(playlist_contents) => {
                        for (info, location) in &playlist_contents {
                            println!("{:?}, {:?}", info, location)
                        }
                    },
                    Err(e) => println!("Failed to show playlist {} with error: {}", playlist_name, e),
                }
            }
            "import_playlist" => {
                let playlist_name = args
                    .get(0)
                    .expect("Usage: import_playlist <playlist>");

                match cache.search_playlist(playlist_name) {
                    Ok(playlist_contents) => {
                        for (info, location) in &playlist_contents {
                            target.import(location, info, Some(PlaylistName::Named(playlist_name.to_string()))).expect("Failed to import_playlist.");
                        }
                    },
                    Err(e) => println!("Failed to import_playlist {} with error: {}", playlist_name, e),
                }
            }
            _ => {}
        }
    }
}
