use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct FileHit {
    pub path: PathBuf,
    pub line: usize,
    pub text: String,
    pub char_index: usize,
}

pub fn system_prefers_dark() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("reg")
            .args([
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "/v",
                "AppsUseLightTheme",
            ])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.contains("0x0") {
                return true;
            }
            if text.contains("0x1") {
                return false;
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            if String::from_utf8_lossy(&output.stdout).to_lowercase().contains("dark") {
                return true;
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(theme) = std::env::var("GTK_THEME") {
            if theme.to_lowercase().contains("dark") {
                return true;
            }
        }
        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            if String::from_utf8_lossy(&output.stdout).contains("dark") {
                return true;
            }
        }
    }
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let hour = (seconds % 86400) / 3600;
    hour < 7 || hour >= 19
}

pub fn today_string() -> String {
    let days = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86400) as i64;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    if month <= 2 {
        year += 1;
    }
    format!("{:04}-{:02}-{:02}", year, month, day)
}

pub fn now_time_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        % 86400;
    format!("{:02}:{:02}:{:02}", seconds / 3600, (seconds / 60) % 60, seconds % 60)
}

pub fn notes_dir(folder: &str) -> PathBuf {
    if !folder.trim().is_empty() {
        return PathBuf::from(folder);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("pezli-editor")
        .join("notes")
}

pub fn ensure_note(path: &Path, template: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if !path.exists() {
        std::fs::write(path, template)?;
    }
    Ok(())
}

pub fn char_index(content: &str, line: usize, col: usize) -> usize {
    let mut current_line = 1usize;
    let mut current_col = 1usize;
    for (index, ch) in content.chars().enumerate() {
        if current_line == line && current_col == col {
            return index;
        }
        if ch == '\n' {
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
    }
    content.chars().count()
}

pub fn line_start_char(content: &str, line: usize) -> usize {
    char_index(content, line, 1)
}

pub fn toggle_checklist(content: &mut String, line: usize) {
    let chars: Vec<char> = content.chars().collect();
    let start = line_start_char(content, line);
    let mut end = start;
    while end < chars.len() && chars[end] != '\n' {
        end += 1;
    }
    let mut text: String = chars[start..end].iter().collect();
    let indent_len = text.len() - text.trim_start().len();
    let indent: String = text.chars().take_while(|c| c.is_whitespace()).collect();
    let body = &text[indent_len..];
    let replacement = if let Some(rest) = body.strip_prefix("- [ ] ") {
        format!("{}- [x] {}", indent, rest)
    } else if let Some(rest) = body.strip_prefix("- [x] ") {
        format!("{}- [ ] {}", indent, rest)
    } else {
        format!("{}- [ ] {}", indent, body)
    };
    text = replacement;
    let mut updated: Vec<char> = Vec::with_capacity(chars.len() - (end - start) + text.len());
    updated.extend_from_slice(&chars[..start]);
    updated.extend(text.chars());
    updated.extend_from_slice(&chars[end..]);
    *content = updated.into_iter().collect();
}

pub fn search_workspace(folders: &[String], needle: &str, max: usize) -> Vec<FileHit> {
    let needle = needle.to_lowercase();
    if needle.trim().is_empty() {
        return Vec::new();
    }
    let skip_dirs = [
        ".git",
        "target",
        "node_modules",
        ".venv",
        "venv",
        "dist",
        "build",
        ".idea",
        ".vscode",
    ];
    let skip_ext = [
        "png", "jpg", "jpeg", "gif", "webp", "pdf", "zip", "tar", "gz", "exe", "so", "dll",
        "bin", "woff", "woff2", "ttf", "ico", "mp3", "mp4", "lock",
    ];
    let mut hits = Vec::new();
    let mut stack: Vec<PathBuf> = folders.iter().map(PathBuf::from).collect();
    'outer: while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
        entries.sort_by_key(|entry| entry.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !skip_dirs.contains(&name) {
                    stack.push(path);
                }
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if skip_ext.contains(&ext.as_str()) {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if meta.len() > 2_000_000 {
                    continue;
                }
            }
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            let mut offset = 0usize;
            for (index, line) in text.lines().enumerate() {
                if line.to_lowercase().contains(&needle) {
                    hits.push(FileHit {
                        path: path.clone(),
                        line: index + 1,
                        text: line.trim().chars().take(160).collect(),
                        char_index: offset,
                    });
                    if hits.len() >= max {
                        break 'outer;
                    }
                }
                offset += line.chars().count() + 1;
            }
        }
    }
    hits
}
