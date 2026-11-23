use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionDoc {
    pub name: String,
    pub path: Option<String>,
    pub content: String,
    pub modified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub docs: Vec<SessionDoc>,
    pub active: usize,
    pub cursor: (usize, usize),
    pub side_view: String,
    pub settings_section: usize,
    pub panel_view: String,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            docs: Vec::new(),
            active: 0,
            cursor: (1, 1),
            side_view: "explorer".to_string(),
            settings_section: 0,
            panel_view: "problems".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
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
    pub side_width: f32,
    pub panel_size: f32,
    pub minimap_width: f32,
    pub custom_themes: Vec<Theme>,
    pub workspace_folders: Vec<String>,
    pub recent_files: Vec<String>,
    pub session: Session,
    pub autosave_secs: u64,
    pub follow_system_theme: bool,
    pub animate_themes: bool,
    pub show_minimap: bool,
    pub show_toasts: bool,
    pub notes_folder: String,
    pub ai_endpoint: String,
    pub ai_model: String,
    pub ai_key: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_id: "dark+".to_string(),
            font_size: 14.0,
            tab_size: 4,
            insert_spaces: true,
            show_line_numbers: true,
            word_wrap: false,
            side_side: "left".to_string(),
            side_visible: true,
            panel_position: "bottom".to_string(),
            panel_visible: true,
            side_width: 290.0,
            panel_size: 220.0,
            minimap_width: 56.0,
            custom_themes: Vec::new(),
            workspace_folders: Vec::new(),
            recent_files: Vec::new(),
            session: Session::default(),
            autosave_secs: 5,
            follow_system_theme: false,
            animate_themes: true,
            show_minimap: true,
            show_toasts: true,
            notes_folder: String::new(),
            ai_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            ai_model: "gpt-4o-mini".to_string(),
            ai_key: String::new(),
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
