use std::fs;
use std::path::Path;

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
        " (Extended)",
    ];

    let mut result = title.to_string();
    for pattern in patterns {
        result = result.replace(pattern, "");
    }

    // Remove "ft. Artist" or "feat. Artist" suffixes
    if let Some(ft_idx) = result.find(" ft. ") {
        result = result[..ft_idx].to_string();
    }
    if let Some(feat_idx) = result.find(" feat. ") {
        result = result[..feat_idx].to_string();
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
    let stem = Path::new(filename).file_stem()?.to_string_lossy();

    let ext = Path::new(filename).extension()?.to_string_lossy();

    // Try splitting on " - " or " – " (en-dash)
    let (artist, title) = stem
        .split_once(" - ")
        .or_else(|| stem.split_once(" – "))?;

    let clean_artist = sanitize_filename(artist.trim());
    let clean_title = sanitize_filename(&normalize_title(title.trim()));

    Some(format!("{} - {}.{}", clean_artist, clean_title, ext))
}

fn main() {
    let cache_dir = Path::new("/Users/aashrayanand/Library/Caches/music-man/audio");

    let entries: Vec<_> = fs::read_dir(cache_dir)
        .expect("Failed to read cache directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| !e.file_name().to_string_lossy().starts_with("._"))
        .collect();

    println!("Found {} files to process\n", entries.len());

    let mut renamed = 0;
    let mut skipped = 0;

    for entry in entries {
        let old_name = entry.file_name().to_string_lossy().to_string();

        if let Some(new_name) = parse_and_rename(&old_name) {
            if old_name != new_name {
                let old_path = entry.path();
                let new_path = cache_dir.join(&new_name);

                println!("Renaming:");
                println!("  FROM: {}", old_name);
                println!("  TO:   {}", new_name);

                match fs::rename(&old_path, &new_path) {
                    Ok(_) => {
                        println!("  ✓ Done\n");
                        renamed += 1;
                    }
                    Err(e) => {
                        println!("  ✗ Error: {}\n", e);
                    }
                }
            } else {
                println!("Already clean: {}\n", old_name);
                skipped += 1;
            }
        } else {
            println!("SKIP (can't parse): {}\n", old_name);
            skipped += 1;
        }
    }

    println!("\nSummary: {} renamed, {} skipped", renamed, skipped);
}
