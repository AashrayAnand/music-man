use std::{path::{Path, PathBuf}, process::Command};
use crate::{AudioError, AudioInfo, audio::AudioLocation};

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
        let dest_file = self.download_audio(full_info.youtube_url.as_ref().expect("Must have youtube URL for download."), &dest)?;
        Ok(AudioLocation::LocalPath(dest_file))
    }
}

impl YtDlpSource {
    fn download_audio(
        &self,
        url: &str,
        output_dir: &Path,
    ) -> Result<PathBuf, AudioError> {
        let dest_file = format!("{}/%(title)s.%(ext)s", output_dir.display());
        let status = Command::new("yt-dlp")
            .args([
                "-x",
                "--audio-format",
                "mp3",
                "--extractor-args", "youtube:player_client=android",
                "-o",
                &dest_file,
                url,
            ])
            .status();
        
        match status {
            Ok(status) => {
                if !status.success() {
                    return Err(AudioError::ExportFailed(format!("ytb-dl exited with status: {}", status)));
                } else {
                    let dest_path = PathBuf::from(&dest_file);
                    if dest_path.exists() {
                        return Ok(dest_path.to_path_buf());
                    } else {
                        return Err(AudioError::ExportFailed(format!("ytb-dl failed to write output file: {}", dest_file)));
                    }
                }
            },
            Err(e) => Err(AudioError::ExportFailed(e.to_string()))
        }
    }

    // Start by connecting song name and artist to youtube, see what we
    // can search by.
    fn search_audio(&self, artist: &str, title: &str) -> Result<String, AudioError> {
        // Trim whitespace and "+" separate name and artist word by word.
        let title_and_artist = title.trim().split_whitespace().chain(artist.trim().split_whitespace()).collect::<Vec<_>>().join(" ");

        let output = Command::new("yt-dlp")
            .args([
                "--get-id",
                "--default-search", "ytsearch1",
                &title_and_artist,
            ])
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    return Err(AudioError::ExportFailed(format!("ytb-dl exited with status: {}", output.status)));
                }
                let video_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(format!("https://www.youtube.com/watch?v={}", video_id))
            },
            Err(e) => Err(AudioError::Io(e))
        }
    }
}