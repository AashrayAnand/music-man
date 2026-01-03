use crate::{AudioError, AudioInfo, audio::AudioLocation};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

// TRAIT: AudioSource, e.g. an open-source mp3 library, an attached drive, the local file cache etc.
// AudioSource impls are able to be read from, and can be used to export music to an AudioTarget:
// 1. search -> Searches the audio device for some audio
// 2. export -> Exports audio from the device to an AudioTarget, using the target's import.
//
// The built-in AudioSource will be the "audio" directory of music-man app's cache directory. We will cache any music we download here.
// Another trait should be implemented by any "audio source".
//
// Other AudioSource could include e.g. ytb-dl based sourcing.
pub trait AudioSource {
    fn name(&self) -> &str;
    fn search(&self, info: &AudioInfo) -> Result<AudioInfo, AudioError>;
    fn fetch(&self, info: &AudioInfo, dest: PathBuf) -> Result<AudioLocation, AudioError>;
}

pub struct YtDlpSource {
    pub name: String,
}

impl AudioSource for YtDlpSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn search(&self, info: &AudioInfo) -> Result<AudioInfo, AudioError> {
        if let (Some(artist), Some(title)) = (&info.artist, &info.title) {
            let url = self.search_audio(title, artist)?;
            let mut extended_info = info.clone();
            extended_info.youtube_url = Some(url);
            return Ok(extended_info);
        }
        Err(AudioError::MissingInfo)
    }

    fn fetch(&self, info: &AudioInfo, dest: PathBuf) -> Result<AudioLocation, AudioError> {
        let full_info = if info.youtube_url.is_some() {
            info
        } else {
            &self.search(info)?
        };
        let dest_file = self.download_audio(full_info, &dest)?;
        Ok(AudioLocation::LocalPath(dest_file))
    }
}

impl YtDlpSource {
    fn download_audio(&self, info: &AudioInfo, output_dir: &Path) -> Result<PathBuf, AudioError> {
        let url = info
            .youtube_url
            .as_ref()
            .expect("Must provide a youtube URL to download audio.");
        let dest_filename = match (&info.artist, &info.title) {
            (Some(artist), Some(title)) => {
                // Use clean file format based on AudioInfo, instead of the URL
                let san_artist = sanitize_filename(artist);
                let san_title = sanitize_filename(title);
                format!(
                    "{}/{} - {}.%(ext)s",
                    output_dir.display(),
                    san_artist,
                    san_title
                )
            }
            _ => format!("{}/%(title)s.%(ext)s", output_dir.display()),
        };
        let status = Command::new("yt-dlp")
            .args([
                "-x",
                "--audio-format",
                "mp3",
                "--extractor-args",
                "youtube:player_client=android",
                "-o",
                &dest_filename,
                url,
            ])
            .status();

        match status {
            Ok(status) => {
                if !status.success() {
                    return Err(AudioError::ExportFailed(format!(
                        "ytb-dl exited with status: {}",
                        status
                    )));
                } else {
                    let dest_path = PathBuf::from(&dest_filename);
                    if dest_path.exists() {
                        return Ok(dest_path.to_path_buf());
                    } else {
                        return Err(AudioError::ExportFailed(format!(
                            "ytb-dl failed to write output file: {}",
                            dest_filename
                        )));
                    }
                }
            }
            Err(e) => Err(AudioError::ExportFailed(e.to_string())),
        }
    }

    // Start by connecting song name and artist to youtube, see what we
    // can search by.
    fn search_audio(&self, artist: &str, title: &str) -> Result<String, AudioError> {
        // Trim whitespace and "+" separate name and artist word by word.
        let title_and_artist = title
            .trim()
            .split_whitespace()
            .chain(artist.trim().split_whitespace())
            .collect::<Vec<_>>()
            .join(" ");

        let output = Command::new("yt-dlp")
            .args([
                "--get-id",
                "--default-search",
                "ytsearch1",
                &title_and_artist,
            ])
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    return Err(AudioError::ExportFailed(format!(
                        "ytb-dl exited with status: {}",
                        output.status
                    )));
                }
                let video_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(format!("https://www.youtube.com/watch?v={}", video_id))
            }
            Err(e) => Err(AudioError::Io(e)),
        }
    }
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

fn normalize_title(title: &str) -> String {
    // Remove common YouTube suffixes
    let patterns = [
        " (Official Video)",
        " (Official Audio)", 
        " (Official Music Video)",
        " [Official Video]",
        " [Official Audio]",
        " (Visualizer)",
        " (Lyrics)",
        " (Audio)",
    ];
    
    let mut result = title.to_string();
    for pattern in patterns {
        result = result.replace(pattern, "");
    }
    
    // Remove video IDs like [ABC123xyz]
    if let Some(bracket_start) = result.rfind(" [") {
        if result.ends_with(']') {
            result = result[..bracket_start].to_string();
        }
    }
    
    result.trim().to_string()
}

fn parse_and_rename(filename: &str) -> Option<String> {
    let stem = Path::new(filename)
        .file_stem()?
        .to_string_lossy();
    
    let ext = Path::new(filename)
        .extension()?
        .to_string_lossy();

    // Try splitting on " - " or " – "
    let (artist, title) = stem.split_once(" - ")
        .or_else(|| stem.split_once(" – "))?;
    
    let clean_artist = sanitize_filename(artist.trim());
    let clean_title = sanitize_filename(&normalize_title(title.trim()));
    
    Some(format!("{} - {}.{}", clean_artist, clean_title, ext))
}
