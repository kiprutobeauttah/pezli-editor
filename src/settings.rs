use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme_id: String,
    pub font_size: f32,
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub side_side: String,
    pub side_visible: bool,
    pub panel_position: String,
    pub panel_visible: bool,
    pub custom_themes: Vec<Theme>,
    pub workspace_folders: Vec<String>,
    pub recent_files: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_id: "light".to_string(),
            font_size: 14.0,
            tab_size: 4,
            insert_spaces: true,
            show_line_numbers: true,
            word_wrap: false,
            side_side: "left".to_string(),
            side_visible: true,
            panel_position: "bottom".to_string(),
            panel_visible: true,
            custom_themes: Vec::new(),
            workspace_folders: Vec::new(),
            recent_files: Vec::new(),
        }
    }
}

fn config_file(name: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".config").join("pezli-editor");
    let _ = std::fs::create_dir_all(&dir);
    dir.join(name)
}

impl Settings {
    pub fn load() -> Self {
        let path = config_file("settings.json");
        match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(config_file("settings.json"), text);
        }
    }
}
