use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub bg: String,
    pub text: String,
    pub chrome: String,
    pub chrome_text: String,
    pub border: String,
    pub hover: String,
    pub accent: String,
    pub selection: String,
    pub line_bg: String,
    pub muted: String,
}

impl Theme {
    pub fn color(&self, hex: &str) -> Color32 {
        hex_color(hex)
    }
}

pub fn hex_color(hex: &str) -> Color32 {
    let h = hex.trim_start_matches('#');
    if h.len() < 6 {
        return Color32::WHITE;
    }
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(255);
    Color32::from_rgb(r, g, b)
}

pub fn color_to_hex(color: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

pub fn builtin_themes() -> Vec<Theme> {
    let entries = [
        ("light", "Light+", "#ffffff", "#1e1e1e", "#f3f3f3", "#252526", "#dcdcdc", "#e8e8e8", "#007acc", "#add6ff", "#f5f5f5", "#6c6c6c"),
        ("dark+", "Dark+", "#1e1e1e", "#d4d4d4", "#252526", "#cccccc", "#3c3c3c", "#2a2d2e", "#007acc", "#264f78", "#282828", "#858585"),
        ("sepia", "Sepia", "#fbf0d9", "#65492c", "#f3e6cc", "#65492c", "#decdae", "#eadcbe", "#a2602c", "#e9d7b9", "#f5e8cf", "#927a5c"),
        ("monokai", "Monokai", "#272822", "#f8f8f2", "#1e1f1c", "#f8f8f2", "#3b3c36", "#34352e", "#a6e22e", "#49483e", "#303127", "#9d9c91"),
        ("solarized-light", "Solarized Light", "#fdf6e3", "#657b83", "#eee8d5", "#586e75", "#d6cfbe", "#e5ddc7", "#268bd2", "#eee8d5", "#f4eedb", "#93a1a1"),
        ("solarized-dark", "Solarized Dark", "#002b36", "#839496", "#073642", "#93a1a1", "#14505b", "#0b3b47", "#2aa198", "#073642", "#05323d", "#657b83"),
        ("dracula", "Dracula", "#282a36", "#f8f8f2", "#21222c", "#f8f8f2", "#3b3d4d", "#353746", "#bd93f9", "#44475a", "#303243", "#a4a7c0"),
        ("nord", "Nord", "#2e3440", "#d8dee9", "#3b4252", "#eceff4", "#4c566a", "#434c5e", "#88c0d0", "#434c5e", "#353c4a", "#a4afc4"),
        ("github-dark", "GitHub Dark", "#0d1117", "#c9d1d9", "#161b22", "#c9d1d9", "#30363d", "#21262d", "#238636", "#1f6feb", "#161b22", "#8b949e"),
        ("one-dark", "One Dark", "#282c34", "#abb2bf", "#21252b", "#abb2bf", "#3b4048", "#2c323c", "#61afef", "#3e4451", "#2d323b", "#7f8490"),
        ("gruvbox", "Gruvbox Dark", "#282828", "#ebdbb2", "#1d2021", "#ebdbb2", "#3c3836", "#32302f", "#fe8019", "#45475a", "#32302f", "#a89984"),
        ("high-contrast", "High Contrast", "#000000", "#ffffff", "#000000", "#ffffff", "#777777", "#333333", "#00ff00", "#0000aa", "#1a1a1a", "#cccccc"),
    ];
    entries
        .iter()
        .map(|(id, name, bg, text, chrome, chrome_text, border, hover, accent, selection, line_bg, muted)| Theme {
            id: id.to_string(),
            name: name.to_string(),
            bg: bg.to_string(),
            text: text.to_string(),
            chrome: chrome.to_string(),
            chrome_text: chrome_text.to_string(),
            border: border.to_string(),
            hover: hover.to_string(),
            accent: accent.to_string(),
            selection: selection.to_string(),
            line_bg: line_bg.to_string(),
            muted: muted.to_string(),
        })
        .collect()
}
