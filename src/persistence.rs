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

#[cfg(test)]
mod test {
    use std::fs;
    use std::path::Path;

    // Helper functions that use a custom path for testing
    fn save_high_score_to(path: &str, score: u32) {
        if let Err(e) = fs::write(path, score.to_string()) {
            eprintln!("Failed to save high score: {}", e);
        }
    }

    fn load_high_score_from(path: &str) -> u32 {
        if Path::new(path).exists() {
            match fs::read_to_string(path) {
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

    fn cleanup(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn save_and_load_high_score() {
        let path = "test_hs_save_load.txt";
        cleanup(path);

        save_high_score_to(path, 42);
        let loaded = load_high_score_from(path);

        assert_eq!(loaded, 42);
        cleanup(path);
    }

    #[test]
    fn load_returns_zero_when_file_missing() {
        let path = "test_hs_missing.txt";
        cleanup(path);
        let loaded = load_high_score_from(path);
        assert_eq!(loaded, 0);
    }

    #[test]
    fn load_returns_zero_for_invalid_content() {
        let path = "test_hs_invalid.txt";
        cleanup(path);
        fs::write(path, "not a number").unwrap();

        let loaded = load_high_score_from(path);
        assert_eq!(loaded, 0);

        cleanup(path);
    }

    #[test]
    fn load_handles_whitespace() {
        let path = "test_hs_whitespace.txt";
        cleanup(path);
        fs::write(path, "  123  \n").unwrap();

        let loaded = load_high_score_from(path);
        assert_eq!(loaded, 123);

        cleanup(path);
    }

    #[test]
    fn save_overwrites_previous_score() {
        let path = "test_hs_overwrite.txt";
        cleanup(path);

        save_high_score_to(path, 100);
        save_high_score_to(path, 200);
        let loaded = load_high_score_from(path);

        assert_eq!(loaded, 200);
        cleanup(path);
    }

    #[test]
    fn save_handles_zero() {
        let path = "test_hs_zero.txt";
        cleanup(path);

        save_high_score_to(path, 0);
        let loaded = load_high_score_from(path);

        assert_eq!(loaded, 0);
        cleanup(path);
    }

    #[test]
    fn save_handles_large_number() {
        let path = "test_hs_large.txt";
        cleanup(path);

        save_high_score_to(path, u32::MAX);
        let loaded = load_high_score_from(path);

        assert_eq!(loaded, u32::MAX);
        cleanup(path);
    }
}
