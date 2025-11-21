use std::fs;
use std::path::Path;

const HIGHSCORE_FILE: &str = "highscore.txt";

pub fn save_high_score(score: u32) {
    if let Err(e) = fs::write(HIGHSCORE_FILE, score.to_string()) {
        eprintln!("Failed to save high score: {}", e);
    }
}

pub fn load_high_score() -> u32 {
    if Path::new(HIGHSCORE_FILE).exists() {
        match fs::read_to_string(HIGHSCORE_FILE) {
            Ok(content) => content.trim().parse().unwrap_or(0),
            Err(e) => {
                eprintln!("Failed to load high score: {}", e);
                0
            }
        }
    } else {
        0
    }
}
