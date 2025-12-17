/// Format bytes to human-readable size
pub fn format_size(n: u64) -> String {
    if n == 0 {
        return "Unknown".to_string();
    }
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = n as f64;
    let mut i = 0;
    while size >= 1024.0 && i < units.len() - 1 {
        size /= 1024.0;
        i += 1;
    }
    format!("{:.2} {}", size, units[i])
}

/// Clean and normalize a PS3 title ID
pub fn clean_title_id(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase()
}

/// Create a safe directory name from a string
pub fn safe_dir_name(raw: &str) -> String {
    // Allow letters, numbers, space, dash, underscore; collapse whitespace
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let cleaned = cleaned
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    if cleaned.is_empty() {
        "PS3Updates".to_string()
    } else {
        cleaned.chars().take(64).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "Unknown");
        assert_eq!(format_size(512), "512.00 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
    }

    #[test]
    fn test_clean_title_id() {
        assert_eq!(clean_title_id("BLES00779"), "BLES00779");
        assert_eq!(clean_title_id("bles-00779"), "BLES00779");
        assert_eq!(clean_title_id("NPUA 80662"), "NPUA80662");
    }

    #[test]
    fn test_safe_dir_name() {
        assert_eq!(safe_dir_name("God of War"), "God of War");
        assert_eq!(safe_dir_name("Game/Title: Test"), "Game Title Test");
        assert_eq!(safe_dir_name(""), "PS3Updates");
    }
}
