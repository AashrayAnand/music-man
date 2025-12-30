use std::path::PathBuf;
use std::fs::create_dir_all;

pub fn setup_app_directories() -> std::io::Result<()> {
    let data_dir = get_data_dir();
    let config_dir = get_config_dir();
    let cache_dir = get_cache_dir();

    // Create base app directories.
    create_dir_all(&data_dir)?;
    create_dir_all(&config_dir)?;
    create_dir_all(&cache_dir)?;

    // Create local file cache for downloaded audio.
    create_dir_all(cache_dir.join("audio"))?;

    println!("Data dir: {:?}", data_dir);
    println!("Cache dir: {:?}", cache_dir);
    println!("Config dir: {:?}", config_dir);
    Ok(())
}

pub fn get_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir()
            .unwrap()
            .join("music-man")
    }
}

pub fn get_config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::config_dir()
            .unwrap()
            .join("music-man")
    }
}

pub fn get_cache_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::cache_dir()
            .unwrap()
            .join("music-man")
    }
}

pub fn local_audio_cache_dir() -> PathBuf {
    get_cache_dir().join("audio")
}