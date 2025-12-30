pub mod app;
pub mod audio;
pub mod index;
pub mod source;
pub mod target;
pub mod device;

use std::{io::stdin, path::{Path, PathBuf}, process::Command};

use crate::{app::{get_cache_dir, local_audio_cache_dir, setup_app_directories}, audio::{AudioError, AudioInfo}, device::AttachedDevice, index::AudioIndex, source::YtDlpSource};

fn main() {
    // Load system state from config + data + cache directories.
    // 1. local file cache, for existing audio.
    // 2. audio lookup map -> mapping (artist, song) -> audio file.
    // 3. 
    setup_app_directories().expect("Failed to create app directories.");

    // 1. Get user device to download audio to.    
    let dirpath = {
        println!("Provide a directory (empty -> {})", local_audio_cache_dir().to_string_lossy());
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

    let source = YtDlpSource {name: "ytdlp".to_string()};
    let cache = AttachedDevice::new(local_audio_cache_dir().display().to_string(), local_audio_cache_dir());
    let target = AttachedDevice::new(dirpath.display().to_string(), dirpath);

    // Iterate sources in order, until we find one that contains the AudioInfo.
    // Fetch from the source to the local file cache, will mean we cache the audio there for a future look up.
    loop {
        print!("> ");
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).expect("Failed to read input");

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
            },
            _ => {},
        }
    }

    // 3. Import music onto device.
    //  - Select a source type for import.
    //  - Load source information e.g. spotify -> get playlists.
    //  - Select from source information e.g. pick a playlist.
    //  - Do import -> create directory on volume + transfer.

    // Initial implementation.
    // 1. Provide a youtube URL.
    // 2. Download YT URL to mp3.
    // 3. Place mp3 onto sp device, with sanitized name.
    /*loop {
        println!("Search (0) or Link (1)?");
        let mut mode = String::new();
        stdin().read_line(&mut mode).expect("Failed to read line");
        let link = match mode.trim() {
            "0" => {
                println!("Provide an artist.");
                let mut artist = String::new();
                stdin().read_line(&mut artist).expect("Failed to read line");
                println!("Provide a song.");
                let mut song = String::new();
                stdin().read_line(&mut song).expect("Failed to read line");
                search_audio(&song, &artist).expect("Failed to search_audio.")
            },
            "1" => {
                println!("Provide a youtube link:");
                let mut link = String::new();
                stdin().read_line(&mut link).expect("Failed to read line");
                link.trim().to_string()
            },
            _ => panic!("Expected mode 0 or 1.")
        };
        println!("Downloading audio from {}", link);

        download_audio(&link, Path::new(&directory)).expect("Failed to download audio");
    }*/
}