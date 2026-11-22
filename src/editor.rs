use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Document {
    pub name: String,
    pub path: Option<PathBuf>,
    pub content: String,
    pub modified: bool,
}

impl Document {
    pub fn untitled(name: String) -> Self {
        Self { name, path: None, content: String::new(), modified: false }
    }

    pub fn from_path(path: PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        Ok(Self { name, path: Some(path), content, modified: false })
    }

    pub fn line_count(&self) -> usize {
        self.content.lines().count().max(1)
    }

    pub fn find_all(&self, needle: &str) -> Vec<(usize, usize)> {
        if needle.is_empty() {
            return Vec::new();
        }
        let chars: Vec<char> = self.content.chars().collect();
        let needle_chars: Vec<char> = needle.chars().collect();
        let mut results = Vec::new();
        if needle_chars.is_empty() {
            return results;
        }
        let mut i = 0;
        while i + needle_chars.len() <= chars.len() {
            if chars[i..i + needle_chars.len()] == needle_chars[..] {
                results.push((i, i + needle_chars.len()));
                i += needle_chars.len();
            } else {
                i += 1;
            }
        }
        results
    }

    pub fn replace_all(&mut self, needle: &str, replacement: &str) -> usize {
        if needle.is_empty() {
            return 0;
        }
        let count = self.find_all(needle).len();
        self.content = self.content.replace(needle, replacement);
        if count > 0 {
            self.modified = true;
        }
        count
    }
}

pub fn char_to_line_col(content: &str, char_index: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut col = 0usize;
    for (index, ch) in content.chars().enumerate() {
        if index >= char_index {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line + 1, col + 1)
}

pub fn char_range_text(content: &str, start: usize, end: usize) -> String {
    content.chars().skip(start).take(end.saturating_sub(start)).collect()
}
