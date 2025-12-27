use core::error;
use std::{io::stdin, path::Path, process::Command};

const SWIM_PRO_PATH: &str = "/Volumes/SWIM PRO";
const SWIM_PRO_MAX_DEPTH: usize = 3;

fn is_supported_audio_file(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_file() {
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

// Download audio from URL with yt-dlp
// Setting up yt-dlp requires:
// 1. ffmpeg
// 2.
fn download_audio(
    url: &str,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // yt-dlp 
    // -v 'https://www.youtube.com/watch?v=KT0B9ArJV0M' --cookies-from-browser brave
    let output_loc = format!("{}/%(title)s.%(ext)s", output_dir.display());
    let dl_cmd = Command::new("yt-dlp")
        .args([
            "-x",
            "--audio-format",
            "mp3",
            //"--audio-quality", "0",
            "--extractor-args", "youtube:player_client=android",
            //"--cookies-from-browser", "brave",
            "-o",
            &output_loc,
            url,
        ])
        .status()
        .expect("Failed to run yt-dlp");

    if !dl_cmd.success() {
        return Err("yt-dlp failed".into());
    }
    Ok(())
}

// Start by connecting song name and artist to youtube, see what we
// can search by.
fn search_audio(name: &str, artist: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Trim whitespace and "+" separate name and artist word by word.
    let name_and_artist = name.trim().split_whitespace().chain(artist.trim().split_whitespace()).collect::<Vec<_>>().join(" ");

    let output = Command::new("yt-dlp")
        .args([
            "--get-id",
            "--default-search", "ytsearch1",
            &name_and_artist,
        ])
        .output()?;
    
    if !output.status.success() {
        return Err("yt-dlp search failed".into())
    }
    let video_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(format!("https://www.youtube.com/watch?v={}", video_id))
}

// You can organize your audio files into different folders for easy classification.
// OpenSwim Pro supports up to 3 levels of folders. Files located in folders beyond this depth may not be recognized.
// With the Shokz App, you can manage the MP3 playback range: play the current folder or play all folders.
// In "Play Current Folder" mode, press and hold the multifunction button and the volume down (–) button for 2 seconds to switch to the next folder.

fn main() {
    // 1. Get SP Device. Whether we need this or not will see.
    let swim_pro = Command::new("diskutil")
        .args(["info", SWIM_PRO_PATH])
        .output()
        .expect("Did not find SWIM PRO device.");
    let _ = String::from_utf8_lossy(&swim_pro.stdout);

    // 2. Get contents from SP device, up to supported directory depth and
    //
    let sp_contents = walkdir::WalkDir::new(SWIM_PRO_PATH)
        .max_depth(SWIM_PRO_MAX_DEPTH)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| is_supported_audio_file(entry));
    for file in sp_contents {
        println!("{:?}", file);
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
    loop {
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

        println!("Provide a directory (empty -> {})", SWIM_PRO_PATH);
        let mut directory = String::new();
        stdin()
            .read_line(&mut directory)
            .expect("Failed to read line");
        let directory = if directory.trim().is_empty() {
            SWIM_PRO_PATH
        } else {
            directory.trim()
        };

        download_audio(&link, Path::new(&directory)).expect("Failed to download audio");
    }
}
