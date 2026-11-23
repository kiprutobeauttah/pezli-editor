use crate::ai::{self, AiReply};
use crate::editor::{char_range_text, char_to_line_col, Document};
use crate::markdown;
use crate::settings::{Session, SessionDoc, Settings};
use crate::theme::{blend_themes, builtin_themes, color_to_hex, hex_color, Theme};
use crate::tools::{self, FileHit};
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SideView {
    Explorer,
    Search,
    Preview,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PanelView {
    Problems,
    Output,
    Ai,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SplitTarget {
    Side,
    Panel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToastKind {
    Info,
    Success,
    Error,
}

struct Toast {
    message: String,
    kind: ToastKind,
    created: Instant,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    NewFile,
    OpenFile,
    OpenFolder,
    Save,
    SaveAs,
    Find,
    ReplaceAll,
    SwitchTheme,
    ToggleSide,
    TogglePanel,
    ToggleWrap,
    ToggleNumbers,
    ToggleMinimap,
    CommandPalette,
    AiAssistant,
    FolderSearch,
    QuickCapture,
    DailyNote,
    InsertTimestamp,
    ToggleChecklist,
    MarkdownPreview,
    About,
    Exit,
    GotoDoc(usize),
}

pub struct PezliApp {
    docs: Vec<Document>,
    active: usize,
    settings: Settings,
    side_view: SideView,
    settings_section: usize,
    panel_view: PanelView,
    find_text: String,
    replace_text: String,
    output: Vec<String>,
    cursor: (usize, usize),
    expanded: HashSet<String>,
    icons: HashMap<String, egui::TextureHandle>,
    settings_dirty: bool,
    about_open: bool,
    editing_theme: Option<String>,
    folder_search: bool,
    folder_hits: Vec<FileHit>,
    selection: String,
    palette_open: bool,
    palette_query: String,
    palette_selected: usize,
    toasts: Vec<Toast>,
    ai_prompt: String,
    ai_response: String,
    show_ai_key: bool,
    ai_busy: bool,
    ai_rx: Option<Receiver<AiReply>>,
    jump_line: Option<usize>,
    quick_open: bool,
    quick_text: String,
    last_autosave: Instant,
    current_theme: Theme,
    theme_from: Theme,
    theme_start: Instant,
    last_theme_id: String,
    system_dark: bool,
    last_system_check: Instant,
    dock_accum: (f32, f32),
    dock_owner: String,
}

fn resolve_theme(settings: &Settings, system_dark: bool) -> Theme {
    let id = if settings.theme_id == "auto" && settings.follow_system_theme {
        if system_dark {
            "dark+"
        } else {
            "light"
        }
    } else {
        settings.theme_id.as_str()
    };
    let mut themes = builtin_themes();
    themes.extend(settings.custom_themes.clone());
    themes
        .into_iter()
        .find(|theme| theme.id == id)
        .unwrap_or_else(|| builtin_themes()[0].clone())
}

fn parse_side_view(value: &str) -> SideView {
    match value {
        "search" => SideView::Search,
        "preview" => SideView::Preview,
        "settings" => SideView::Settings,
        _ => SideView::Explorer,
    }
}

fn parse_panel_view(value: &str) -> PanelView {
    match value {
        "output" => PanelView::Output,
        "ai" => PanelView::Ai,
        _ => PanelView::Problems,
    }
}

impl PezliApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let mut icons = HashMap::new();
        for (name, bytes) in crate::icons_data::icon_bytes() {
            let Ok(decoded) = image::load_from_memory(bytes) else { continue };
            let decoded = decoded.to_rgba8();
            let samples = decoded.as_flat_samples();
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [decoded.width() as usize, decoded.height() as usize],
                samples.as_slice(),
            );
            let texture = cc.egui_ctx.load_texture(
                name.to_string(),
                image,
                egui::TextureOptions::LINEAR,
            );
            icons.insert(name.to_string(), texture);
        }

        let session = settings.session.clone();
        let mut docs = Vec::new();
        for item in &session.docs {
            let mut doc = match &item.path {
                Some(path) => Document::from_path(PathBuf::from(path))
                    .unwrap_or_else(|_| Document::untitled(item.name.clone())),
                None => Document::untitled(item.name.clone()),
            };
            doc.content = item.content.clone();
            doc.modified = item.modified;
            docs.push(doc);
        }
        if docs.is_empty() {
            docs.push(Document::untitled("Untitled-1".to_string()));
        }
        let active = session.active.min(docs.len() - 1);
        let system_dark = if settings.follow_system_theme {
            tools::system_prefers_dark()
        } else {
            false
        };
        let theme = resolve_theme(&settings, system_dark);

        Self {
            docs,
            active,
            last_theme_id: theme.id.clone(),
            theme_from: theme.clone(),
            current_theme: theme,
            theme_start: Instant::now(),
            system_dark,
            last_system_check: Instant::now(),
            dock_accum: (0.0, 0.0),
            dock_owner: String::new(),
            settings,
            side_view: parse_side_view(&session.side_view),
            settings_section: session.settings_section,
            panel_view: parse_panel_view(&session.panel_view),
            find_text: String::new(),
            replace_text: String::new(),
            output: vec!["Pezli Editor ready".to_string()],
            cursor: session.cursor,
            expanded: HashSet::new(),
            icons,
            settings_dirty: false,
            about_open: false,
            editing_theme: None,
            folder_search: false,
            folder_hits: Vec::new(),
            selection: String::new(),
            palette_open: false,
            palette_query: String::new(),
            palette_selected: 0,
            toasts: Vec::new(),
            ai_prompt: String::new(),
            ai_response: String::new(),
            show_ai_key: false,
            ai_busy: false,
            ai_rx: None,
            jump_line: None,
            quick_open: false,
            quick_text: String::new(),
            last_autosave: Instant::now(),
        }
    }

    fn all_themes(&self) -> Vec<Theme> {
        let mut themes = builtin_themes();
        themes.extend(self.settings.custom_themes.clone());
        themes
    }

    fn target_theme(&self) -> Theme {
        resolve_theme(&self.settings, self.system_dark)
    }

    fn theme(&self) -> Theme {
        self.current_theme.clone()
    }

    fn capture_session(&mut self) {
        self.settings.session = Session {
            docs: self
                .docs
                .iter()
                .map(|doc| SessionDoc {
                    name: doc.name.clone(),
                    path: doc.path.as_ref().map(|p| p.to_string_lossy().into_owned()),
                    content: doc.content.clone(),
                    modified: doc.modified,
                })
                .collect(),
            active: self.active,
            cursor: self.cursor,
            side_view: match self.side_view {
                SideView::Explorer => "explorer",
                SideView::Search => "search",
                SideView::Preview => "preview",
                SideView::Settings => "settings",
            }
            .to_string(),
            settings_section: self.settings_section,
            panel_view: match self.panel_view {
                PanelView::Problems => "problems",
                PanelView::Output => "output",
                PanelView::Ai => "ai",
            }
            .to_string(),
        };
    }

    fn toast(&mut self, message: impl Into<String>, kind: ToastKind) {
        if !self.settings.show_toasts {
            return;
        }
        self.toasts.push(Toast {
            message: message.into(),
            kind,
            created: Instant::now(),
        });
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
    }

    fn log(&mut self, message: impl Into<String>) {
        self.output.push(message.into());
        if self.output.len() > 200 {
            self.output.remove(0);
        }
    }

    fn apply_style(&mut self, ctx: &egui::Context) {
        let target = self.target_theme();
        if target.id != self.last_theme_id {
            self.theme_from = self.current_theme.clone();
            self.last_theme_id = target.id.clone();
            self.theme_start = Instant::now();
        }
        let progress = if self.settings.animate_themes {
            (self.theme_start.elapsed().as_secs_f32() / 0.28).min(1.0)
        } else {
            1.0
        };
        self.current_theme = if progress >= 1.0 {
            target
        } else {
            blend_themes(&self.theme_from, &target, progress)
        };
        if progress < 1.0 {
            ctx.request_repaint();
        }

        let theme = self.current_theme.clone();
        let chrome = hex_color(&theme.chrome);
        let bg = hex_color(&theme.bg);
        let text = hex_color(&theme.text);
        let muted = hex_color(&theme.muted);
        let accent = hex_color(&theme.accent);
        let hover = hex_color(&theme.hover);

        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = chrome;
        visuals.panel_fill = bg;
        visuals.override_text_color = Some(text);
        visuals.window_shadow.color = egui::Color32::from_black_alpha(90);
        visuals.popup_shadow.color = egui::Color32::from_black_alpha(90);

        let radius = egui::CornerRadius::same(5);
        visuals.widgets.noninteractive.fg_stroke.color = muted;
        visuals.widgets.noninteractive.bg_fill = chrome;
        visuals.widgets.noninteractive.corner_radius = radius;

        visuals.widgets.inactive.fg_stroke.color = hex_color(&theme.chrome_text);
        visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.inactive.bg_fill = chrome;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, hex_color(&theme.border));
        visuals.widgets.inactive.corner_radius = radius;

        visuals.widgets.hovered.fg_stroke.color = hex_color(&theme.chrome_text);
        visuals.widgets.hovered.weak_bg_fill = hover;
        visuals.widgets.hovered.bg_fill = hover;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, hex_color(&theme.accent));
        visuals.widgets.hovered.corner_radius = radius;

        visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;
        visuals.widgets.active.weak_bg_fill = accent;
        visuals.widgets.active.bg_fill = accent;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, accent);
        visuals.widgets.active.corner_radius = radius;

        visuals.selection.bg_fill = hex_color(&theme.selection);
        visuals.selection.stroke.color = accent;
        visuals.hyperlink_color = accent;
        visuals.faint_bg_color = hex_color(&theme.line_bg);
        ctx.set_visuals(visuals);

        let font_size = self.settings.font_size;
        ctx.all_styles_mut(|style| {
            for font in style.text_styles.values_mut() {
                font.size = font_size;
            }
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(5);
            style.spacing.button_padding = egui::vec2(8.0, 4.0);
            style.spacing.item_spacing = egui::vec2(8.0, 5.0);
        });
    }

    fn icon(&self, ui: &mut egui::Ui, name: &str, size: f32, color: egui::Color32) {
        if let Some(texture) = self.icons.get(name) {
            let tex_size = texture.size();
            let sized = egui::load::SizedTexture::new(
                texture.id(),
                egui::vec2(tex_size[0] as f32, tex_size[1] as f32),
            );
            ui.add(
                egui::Image::new(sized)
                    .fit_to_exact_size(egui::vec2(size, size))
                    .tint(color),
            );
        }
    }

    fn icon_button(&self, ui: &mut egui::Ui, name: &str, size: f32, color: egui::Color32) -> bool {
        if let Some(texture) = self.icons.get(name) {
            let tex_size = texture.size();
            let sized = egui::load::SizedTexture::new(
                texture.id(),
                egui::vec2(tex_size[0] as f32, tex_size[1] as f32),
            );
            let image = egui::Image::new(sized)
                .fit_to_exact_size(egui::vec2(size, size))
                .tint(color);
            return ui.add(egui::Button::image(image).frame(false)).clicked();
        }
        false
    }

    fn dock_grip(&mut self, ui: &mut egui::Ui, owner: &str) -> Option<(f32, f32)> {
        let accent = hex_color(&self.current_theme.accent);
        let muted = hex_color(&self.current_theme.muted);
        let active = self.dock_owner == owner;
        let grip = ui.add(
            egui::Label::new(egui::RichText::new("::").size(13.0).color(if active { accent } else { muted }))
                .sense(egui::Sense::drag()),
        );
        if grip.drag_started() {
            self.dock_accum = (0.0, 0.0);
            self.dock_owner = owner.to_string();
        }
        if grip.dragged() {
            let delta = grip.drag_delta();
            self.dock_accum.0 += delta.x;
            self.dock_accum.1 += delta.y;
        }
        if grip.drag_stopped() && self.dock_owner == owner {
            let result = self.dock_accum;
            self.dock_accum = (0.0, 0.0);
            self.dock_owner.clear();
            return Some(result);
        }
        None
    }

    fn resize_split(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        vertical: bool,
        target: SplitTarget,
        sign: f32,
        id: &str,
    ) {
        let response = ui.interact(rect, ui.id().with(id), egui::Sense::drag());
        if response.dragged() {
            let delta = response.drag_delta();
            if vertical {
                self.settings.panel_size = (self.settings.panel_size - delta.y).clamp(120.0, 600.0);
            } else {
                match target {
                    SplitTarget::Side => {
                        self.settings.side_width =
                            (self.settings.side_width + delta.x * sign).clamp(180.0, 560.0);
                    }
                    SplitTarget::Panel => {
                        self.settings.panel_size =
                            (self.settings.panel_size + delta.x * sign).clamp(180.0, 640.0);
                    }
                }
            }
            self.settings_dirty = true;
        }
        let accent = hex_color(&self.current_theme.accent);
        let border = hex_color(&self.current_theme.border);
        let color = if response.hovered() || response.dragged() {
            accent
        } else {
            border
        };
        ui.painter().rect_filled(rect, 2.0, color);
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(if vertical {
                egui::CursorIcon::ResizeVertical
            } else {
                egui::CursorIcon::ResizeHorizontal
            });
        }
    }

    fn ui_layout_splitters(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let thickness = 5.0;
        if self.settings.side_visible {
            let right = self.settings.side_side == "right";
            let x = if right {
                rect.right() - thickness
            } else {
                rect.left()
            };
            let handle = egui::Rect::from_min_size(
                egui::pos2(x, rect.top()),
                egui::vec2(thickness, rect.height()),
            );
            self.resize_split(ui, handle, false, SplitTarget::Side, if right { -1.0 } else { 1.0 }, "side_split");
        }
        if self.settings.panel_visible {
            let position = self.settings.panel_position.as_str();
            let handle = match position {
                "bottom" => egui::Rect::from_min_size(
                    egui::pos2(rect.left(), rect.bottom() - thickness),
                    egui::vec2(rect.width(), thickness),
                ),
                "left" => egui::Rect::from_min_size(
                    egui::pos2(rect.left(), rect.top()),
                    egui::vec2(thickness, rect.height()),
                ),
                "right" => egui::Rect::from_min_size(
                    egui::pos2(rect.right() - thickness, rect.top()),
                    egui::vec2(thickness, rect.height()),
                ),
                _ => egui::Rect::NOTHING,
            };
            let sign = if position == "right" { -1.0 } else { 1.0 };
            self.resize_split(ui, handle, position == "bottom", SplitTarget::Panel, sign, "panel_split");
        }
    }

    fn new_doc(&mut self) {
        let name = format!("Untitled-{}", self.docs.len() + 1);
        self.docs.push(Document::untitled(name));
        self.active = self.docs.len() - 1;
        self.cursor = (1, 1);
    }

    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.open_path(path);
        }
    }

    fn open_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            let text = path.to_string_lossy().into_owned();
            if !self.settings.workspace_folders.contains(&text) {
                self.settings.workspace_folders.push(text);
                self.settings_dirty = true;
            }
            self.log(format!("Opened folder: {}", path.display()));
            self.toast(format!("Workspace: {}", path.display()), ToastKind::Success);
        }
    }

    fn open_path(&mut self, path: PathBuf) {
        if let Some(index) = self.docs.iter().position(|doc| doc.path.as_ref() == Some(&path)) {
            self.active = index;
            return;
        }
        match Document::from_path(path.clone()) {
            Ok(doc) => {
                self.docs.push(doc);
                self.active = self.docs.len() - 1;
                self.cursor = (1, 1);
                self.log(format!("Opened: {}", path.display()));
                self.toast(format!("Opened {}", path.display()), ToastKind::Success);
                let text = path.to_string_lossy().into_owned();
                self.settings.recent_files.retain(|item| item != &text);
                self.settings.recent_files.insert(0, text);
                self.settings.recent_files.truncate(12);
                self.settings_dirty = true;
            }
            Err(error) => {
                self.log(format!("Failed to open file: {}", error));
                self.toast(format!("Cannot open file: {}", error), ToastKind::Error);
            }
        }
    }

    fn save_file(&mut self, save_as: bool) {
        let path = if save_as || self.docs[self.active].path.is_none() {
            rfd::FileDialog::new().save_file()
        } else {
            self.docs[self.active].path.clone()
        };
        let Some(path) = path else { return };
        let content = self.docs[self.active].content.clone();
        match std::fs::write(&path, content) {
            Ok(()) => {
                self.docs[self.active].path = Some(path.clone());
                self.docs[self.active].name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.to_string_lossy().into_owned());
                self.docs[self.active].modified = false;
                self.log(format!("Saved: {}", path.display()));
                self.toast(format!("Saved {}", self.docs[self.active].name), ToastKind::Success);
            }
            Err(error) => {
                self.log(format!("Failed to save: {}", error));
                self.toast(format!("Save failed: {}", error), ToastKind::Error);
            }
        }
    }

    fn do_find(&mut self) {
        let results = self.docs[self.active].find_all(&self.find_text);
        self.log(format!("Found {} matches", results.len()));
        if self.folder_search {
            self.folder_hits = tools::search_workspace(&self.settings.workspace_folders, &self.find_text, 300);
            self.toast(
                format!("{} matches across workspace files", self.folder_hits.len()),
                ToastKind::Info,
            );
        }
    }

    fn do_replace(&mut self) {
        let count = self.docs[self.active].replace_all(&self.find_text, &self.replace_text);
        self.log(format!("Replaced {} occurrences", count));
        self.toast(format!("Replaced {} occurrences", count), ToastKind::Success);
    }

    fn next_theme(&mut self) {
        let themes = self.all_themes();
        let index = themes
            .iter()
            .position(|theme| theme.id == self.settings.theme_id)
            .unwrap_or(0);
        self.settings.theme_id = themes[(index + 1) % themes.len()].id.clone();
        self.settings_dirty = true;
    }

    fn insert_text(&mut self, text: &str) {
        let doc = &mut self.docs[self.active];
        let index = tools::char_index(&doc.content, self.cursor.0, self.cursor.1);
        let chars: Vec<char> = doc.content.chars().collect();
        let mut updated: Vec<char> = Vec::with_capacity(chars.len() + text.len());
        updated.extend_from_slice(&chars[..index]);
        updated.extend(text.chars());
        updated.extend_from_slice(&chars[index..]);
        doc.content = updated.into_iter().collect();
        doc.modified = true;
    }

    fn toggle_current_checklist(&mut self) {
        let doc = &mut self.docs[self.active];
        tools::toggle_checklist(&mut doc.content, self.cursor.0);
        doc.modified = true;
    }

    fn new_daily_note(&mut self) {
        let date = tools::today_string();
        let path = tools::notes_dir(&self.settings.notes_folder).join(format!("{}.md", date));
        let template = format!(
            "# {}\n\n## Focus\n\n- [ ] \n\n## Notes\n\n\n## Links\n\n",
            date
        );
        match tools::ensure_note(&path, &template) {
            Ok(()) => {
                self.open_path(path);
                self.toast("Daily note ready", ToastKind::Success);
            }
            Err(error) => self.toast(format!("Cannot create note: {}", error), ToastKind::Error),
        }
    }

    fn save_quick_capture(&mut self) {
        let text = self.quick_text.trim().to_string();
        if text.is_empty() {
            return;
        }
        let path = tools::notes_dir(&self.settings.notes_folder).join("quick-capture.md");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let entry = format!("\n## {} — {}\n\n{}\n", tools::today_string(), tools::now_time_string(), text);
        let previous = std::fs::read_to_string(&path).unwrap_or_default();
        match std::fs::write(&path, format!("{}{}", previous, entry)) {
            Ok(()) => {
                self.quick_text.clear();
                self.quick_open = false;
                self.toast("Thought captured", ToastKind::Success);
                self.open_path(path);
            }
            Err(error) => self.toast(format!("Capture failed: {}", error), ToastKind::Error),
        }
    }

    fn ask_ai(&mut self, instruction: &str) {
        if self.ai_busy {
            return;
        }
        if self.settings.ai_key.trim().is_empty() {
            self.ai_response =
                "Add your API key in Settings → AI, then try again.".to_string();
            self.toast("AI needs an API key", ToastKind::Error);
            return;
        }
        let doc = &self.docs[self.active];
        let context = if self.selection.trim().is_empty() {
            format!(
                "File: {}\n\n{}",
                doc.name,
                doc.content.chars().take(8000).collect::<String>()
            )
        } else {
            format!("File: {}\n\nSelected text:\n{}", doc.name, self.selection)
        };
        let (tx, rx) = mpsc::channel();
        self.ai_rx = Some(rx);
        self.ai_busy = true;
        let endpoint = self.settings.ai_endpoint.clone();
        let model = self.settings.ai_model.clone();
        let key = self.settings.ai_key.clone();
        let instruction = instruction.to_string();
        std::thread::spawn(move || {
            let _ = tx.send(ai::ask(&endpoint, &model, &key, &instruction, &context));
        });
    }

    fn poll_ai(&mut self) {
        let mut reply = None;
        let mut disconnected = false;
        if let Some(rx) = &self.ai_rx {
            match rx.try_recv() {
                Ok(result) => reply = Some(result),
                Err(TryRecvError::Disconnected) => disconnected = true,
                Err(TryRecvError::Empty) => {}
            }
        }
        if reply.is_some() || disconnected {
            self.ai_rx = None;
            self.ai_busy = false;
        }
        if let Some(result) = reply {
            self.ai_response = result.text;
            self.toast(
                if result.ok { "AI response ready" } else { "AI request failed" },
                if result.ok { ToastKind::Success } else { ToastKind::Error },
            );
        }
        if disconnected {
            self.ai_response = "The AI worker stopped unexpectedly.".to_string();
        }
    }

    fn commands(&self) -> Vec<(String, String, Command)> {
        let mut list = vec![
            ("New File".to_string(), "Ctrl+N".to_string(), Command::NewFile),
            ("Open File".to_string(), "Ctrl+O".to_string(), Command::OpenFile),
            ("Open Folder".to_string(), "workspace".to_string(), Command::OpenFolder),
            ("Save".to_string(), "Ctrl+S".to_string(), Command::Save),
            ("Save As".to_string(), "Ctrl+Shift+S".to_string(), Command::SaveAs),
            ("Find and Replace".to_string(), "Ctrl+F".to_string(), Command::Find),
            ("Replace All".to_string(), "search".to_string(), Command::ReplaceAll),
            ("Search Workspace Files".to_string(), "Ctrl+Shift+F".to_string(), Command::FolderSearch),
            ("Command Palette".to_string(), "Ctrl+P".to_string(), Command::CommandPalette),
            ("Switch Theme".to_string(), "Ctrl+T".to_string(), Command::SwitchTheme),
            ("Toggle Side Bar".to_string(), "Ctrl+B".to_string(), Command::ToggleSide),
            ("Toggle Panel".to_string(), "view".to_string(), Command::TogglePanel),
            ("Toggle Word Wrap".to_string(), "editor".to_string(), Command::ToggleWrap),
            ("Toggle Line Numbers".to_string(), "editor".to_string(), Command::ToggleNumbers),
            ("Toggle Minimap".to_string(), "view".to_string(), Command::ToggleMinimap),
            ("Markdown Preview".to_string(), "Ctrl+Shift+M".to_string(), Command::MarkdownPreview),
            ("AI Assistant".to_string(), "Ctrl+Alt+A".to_string(), Command::AiAssistant),
            ("Quick Capture".to_string(), "Ctrl+Shift+Q".to_string(), Command::QuickCapture),
            ("New Daily Note".to_string(), "Ctrl+Shift+D".to_string(), Command::DailyNote),
            ("Insert Timestamp".to_string(), "notes".to_string(), Command::InsertTimestamp),
            ("Toggle Checklist".to_string(), "notes".to_string(), Command::ToggleChecklist),
            ("About Pezli".to_string(), "help".to_string(), Command::About),
            ("Exit".to_string(), "quit".to_string(), Command::Exit),
        ];
        for (index, doc) in self.docs.iter().enumerate() {
            list.push((
                format!("Go to {}", doc.name),
                if index == self.active { "current" } else { "tab" }.to_string(),
                Command::GotoDoc(index),
            ));
        }
        list
    }

    fn run_command(&mut self, ctx: &egui::Context, command: Command) {
        match command {
            Command::NewFile => self.new_doc(),
            Command::OpenFile => self.open_file(),
            Command::OpenFolder => self.open_folder(),
            Command::Save => self.save_file(false),
            Command::SaveAs => self.save_file(true),
            Command::Find => {
                self.side_view = SideView::Search;
                self.settings.side_visible = true;
                self.settings_dirty = true;
            }
            Command::ReplaceAll => self.do_replace(),
            Command::SwitchTheme => self.next_theme(),
            Command::ToggleSide => {
                self.settings.side_visible = !self.settings.side_visible;
                self.settings_dirty = true;
            }
            Command::TogglePanel => {
                self.settings.panel_visible = !self.settings.panel_visible;
                self.settings_dirty = true;
            }
            Command::ToggleWrap => {
                self.settings.word_wrap = !self.settings.word_wrap;
                self.settings_dirty = true;
            }
            Command::ToggleNumbers => {
                self.settings.show_line_numbers = !self.settings.show_line_numbers;
                self.settings_dirty = true;
            }
            Command::ToggleMinimap => {
                self.settings.show_minimap = !self.settings.show_minimap;
                self.settings_dirty = true;
            }
            Command::CommandPalette => self.palette_open = true,
            Command::AiAssistant => {
                self.panel_view = PanelView::Ai;
                self.settings.panel_visible = true;
                self.settings_dirty = true;
            }
            Command::FolderSearch => {
                self.side_view = SideView::Search;
                self.folder_search = true;
                self.settings.side_visible = true;
                self.settings_dirty = true;
            }
            Command::QuickCapture => self.quick_open = true,
            Command::DailyNote => self.new_daily_note(),
            Command::InsertTimestamp => {
                self.insert_text(&format!("[{}] ", tools::now_time_string()))
            }
            Command::ToggleChecklist => self.toggle_current_checklist(),
            Command::MarkdownPreview => {
                self.side_view = if self.side_view == SideView::Preview {
                    SideView::Explorer
                } else {
                    SideView::Preview
                };
                self.settings.side_visible = true;
                self.settings_dirty = true;
            }
            Command::About => self.about_open = true,
            Command::Exit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Command::GotoDoc(index) => {
                if index < self.docs.len() {
                    self.active = index;
                }
            }
        }
    }

    fn ui_menu(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme();
        let chrome = hex_color(&theme.chrome);
        let accent = hex_color(&theme.accent);
        let muted = hex_color(&theme.muted);
        let theme_name = theme.name.clone();
        egui::Panel::top("menu_bar")
            .frame(
                egui::Frame::new()
                    .fill(chrome)
                    .inner_margin(egui::Margin::symmetric(10, 5)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New File").clicked() { self.new_doc(); }
                        if ui.button("Open File...").clicked() { self.open_file(); }
                        if ui.button("Open Folder...").clicked() { self.open_folder(); }
                        ui.separator();
                        if ui.button("Save").clicked() { self.save_file(false); }
                        if ui.button("Save As...").clicked() { self.save_file(true); }
                        ui.separator();
                        if ui.button("Exit").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui.button("Command Palette...   Ctrl+P").clicked() {
                            self.palette_open = true;
                        }
                        if ui.button("Find and Replace...").clicked() {
                            self.side_view = SideView::Search;
                            self.settings.side_visible = true;
                        }
                        if ui.button("Replace All").clicked() { self.do_replace(); }
                        ui.separator();
                        if ui.button("Insert Timestamp").clicked() {
                            self.insert_text(&format!("[{}] ", tools::now_time_string()));
                        }
                        if ui.button("Toggle Checklist").clicked() {
                            self.toggle_current_checklist();
                        }
                    });
                    ui.menu_button("Notes", |ui| {
                        if ui.button("Quick Capture...   Ctrl+Shift+Q").clicked() {
                            self.quick_open = true;
                        }
                        if ui.button("New Daily Note   Ctrl+Shift+D").clicked() {
                            self.new_daily_note();
                        }
                        if ui.button("Markdown Preview   Ctrl+Shift+M").clicked() {
                            self.side_view = SideView::Preview;
                            self.settings.side_visible = true;
                        }
                        if ui.button("Search Workspace Files").clicked() {
                            self.side_view = SideView::Search;
                            self.folder_search = true;
                            self.settings.side_visible = true;
                        }
                    });
                    ui.menu_button("View", |ui| {
                        if ui.checkbox(&mut self.settings.side_visible, "Side Bar").clicked() {
                            self.settings_dirty = true;
                        }
                        if ui.checkbox(&mut self.settings.panel_visible, "Panel").clicked() {
                            self.settings_dirty = true;
                        }
                        if ui.checkbox(&mut self.settings.show_minimap, "Minimap").clicked() {
                            self.settings_dirty = true;
                        }
                        ui.separator();
                        if ui.button("Switch Theme").clicked() { self.next_theme(); }
                        if ui.button("Settings").clicked() {
                            self.side_view = SideView::Settings;
                            self.settings.side_visible = true;
                            self.settings_dirty = true;
                        }
                    });
                    ui.menu_button("AI", |ui| {
                        if ui.button("Open Assistant   Ctrl+Alt+A").clicked() {
                            self.panel_view = PanelView::Ai;
                            self.settings.panel_visible = true;
                        }
                        if ui.button("Explain Selection").clicked() {
                            self.ask_ai("Explain the supplied text clearly and briefly.");
                            self.panel_view = PanelView::Ai;
                            self.settings.panel_visible = true;
                        }
                        if ui.button("Improve Writing").clicked() {
                            self.ask_ai("Rewrite the supplied text to be clearer and more concise. Return only the improved text.");
                            self.panel_view = PanelView::Ai;
                            self.settings.panel_visible = true;
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("Keyboard Shortcuts").clicked() {
                            self.log("Shortcuts: Ctrl+P commands, Ctrl+N new, Ctrl+O open, Ctrl+S save, Ctrl+F find, Ctrl+B side bar, Ctrl+T theme, Ctrl+Shift+Q capture, Ctrl+Shift+D daily note, Ctrl+Shift+M preview, Ctrl+Alt+A AI");
                        }
                        if ui.button("About").clicked() { self.about_open = true; }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("BK")
                                .color(egui::Color32::WHITE)
                                .background_color(accent),
                        );
                        ui.label(egui::RichText::new("Beauttah Kipruto").color(muted).size(11.0));
                        ui.label(egui::RichText::new("|").color(muted));
                        ui.label(egui::RichText::new(theme_name).color(accent));
                        ui.label(
                            egui::RichText::new("P")
                                .color(egui::Color32::WHITE)
                                .background_color(accent)
                                .size(16.0),
                        );
                    });
                });
            });
    }

    fn ui_toolbar(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme();
        let chrome = hex_color(&theme.chrome);
        let border = hex_color(&theme.border);
        let accent = hex_color(&theme.accent);
        egui::Panel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(chrome)
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .stroke(egui::Stroke::new(1.0, border)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.icon_button(ui, "file-plus", 18.0, accent) { self.new_doc(); }
                    if self.icon_button(ui, "folder", 18.0, accent) { self.open_file(); }
                    if self.icon_button(ui, "save", 18.0, accent) { self.save_file(false); }
                    ui.separator();
                    if self.icon_button(ui, "search", 18.0, accent) {
                        self.side_view = SideView::Search;
                        self.settings.side_visible = true;
                    }
                    if self.icon_button(ui, "refresh-cw", 18.0, accent) { self.do_replace(); }
                    ui.separator();
                    if self.icon_button(ui, "droplet", 18.0, accent) { self.next_theme(); }
                    if self.icon_button(ui, "settings", 18.0, accent) {
                        self.side_view = SideView::Settings;
                        self.settings.side_visible = true;
                    }
                    ui.separator();
                    if ui.button("Commands").clicked() { self.palette_open = true; }
                    if ui.button("✎ Capture").clicked() { self.quick_open = true; }
                    if ui.button("AI").clicked() {
                        self.panel_view = PanelView::Ai;
                        self.settings.panel_visible = true;
                    }
                });
            });
    }

    fn ui_tabs(&mut self, ui: &mut egui::Ui) {
        let chrome = hex_color(&self.theme().chrome);
        let border = hex_color(&self.theme().border);
        let accent = hex_color(&self.theme().accent);
        let muted = hex_color(&self.theme().muted);
        egui::Panel::top("tabs_bar")
            .frame(
                egui::Frame::new()
                    .fill(chrome)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .stroke(egui::Stroke::new(1.0, border)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for index in 0..self.docs.len() {
                        let name = if self.docs[index].modified {
                            format!("{} •", self.docs[index].name)
                        } else {
                            self.docs[index].name.clone()
                        };
                        let selected = index == self.active;
                        ui.horizontal(|ui| {
                            if let Some((dx, _)) = self.dock_grip(ui, &format!("tab-{}", index)) {
                                let steps = (dx / 110.0).round() as i32;
                                if steps != 0 {
                                    let target = (index as i32 + steps)
                                        .clamp(0, self.docs.len() as i32 - 1)
                                        as usize;
                                    let doc = self.docs.remove(index);
                                    self.docs.insert(target, doc);
                                    if self.active == index {
                                        self.active = target;
                                    } else if index < self.active && target >= self.active {
                                        self.active -= 1;
                                    } else if index > self.active && target <= self.active {
                                        self.active += 1;
                                    }
                                }
                            }
                            self.icon(ui, "file", 14.0, if selected { accent } else { muted });
                            if ui.selectable_label(selected, name).clicked() {
                                self.active = index;
                            }
                        });
                        if ui.small_button("×").clicked() && self.docs.len() > 1 {
                            self.docs.remove(index);
                            if self.active >= index && self.active > 0 {
                                self.active -= 1;
                            }
                            self.active = self.active.min(self.docs.len() - 1);
                            break;
                        }
                    }
                    if ui.small_button("+").clicked() {
                        self.new_doc();
                    }
                });
            });
    }

    fn ui_side(&mut self, ui: &mut egui::Ui) {
        if !self.settings.side_visible {
            return;
        }
        let side_right = self.settings.side_side == "right";
        let build = |ui: &mut egui::Ui, app: &mut Self| {
            ui.horizontal(|ui| {
                if ui.selectable_label(app.side_view == SideView::Explorer, "Files").clicked() {
                    app.side_view = SideView::Explorer;
                }
                if ui.selectable_label(app.side_view == SideView::Search, "Search").clicked() {
                    app.side_view = SideView::Search;
                }
                if ui.selectable_label(app.side_view == SideView::Preview, "Preview").clicked() {
                    app.side_view = SideView::Preview;
                }
                if ui.selectable_label(app.side_view == SideView::Settings, "Settings").clicked() {
                    app.side_view = SideView::Settings;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some((dx, _)) = app.dock_grip(ui, "side") {
                        if dx > 40.0 {
                            app.settings.side_side = "right".into();
                        } else if dx < -40.0 {
                            app.settings.side_side = "left".into();
                        }
                        app.settings_dirty = true;
                    }
                });
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| match app.side_view {
                SideView::Explorer => app.ui_explorer(ui),
                SideView::Search => app.ui_search(ui),
                SideView::Preview => {
                    let accent = hex_color(&app.current_theme.accent);
                    let mut content = app.docs[app.active].content.clone();
                    let font_size = app.settings.font_size;
                    markdown::render_markdown(ui, &mut content, font_size, accent);
                    if content != app.docs[app.active].content {
                        app.docs[app.active].content = content;
                        app.docs[app.active].modified = true;
                    }
                }
                SideView::Settings => {
                    let sections = [
                        "General",
                        "Editor",
                        "Appearance",
                        "Themes",
                        "Workspace",
                        "AI",
                        "Notes",
                    ];
                    for (index, name) in sections.iter().enumerate() {
                        if ui.selectable_label(app.settings_section == index, *name).clicked() {
                            app.settings_section = index;
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Settings are shown in the editor area").weak());
                }
            });
        };

        let chrome = hex_color(&self.theme().chrome);
        let border = hex_color(&self.theme().border);
        let frame = || {
            egui::Frame::new()
                .fill(chrome)
                .inner_margin(egui::Margin::symmetric(10, 8))
                .stroke(egui::Stroke::new(1.0, border))
        };
        let width = self.settings.side_width;
        if side_right {
            egui::Panel::right("side_panel")
                .exact_size(width)
                .frame(frame())
                .show(ui, |ui| build(ui, self));
        } else {
            egui::Panel::left("side_panel")
                .exact_size(width)
                .frame(frame())
                .show(ui, |ui| build(ui, self));
        }
    }

    fn ui_explorer(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("OPEN EDITORS", |ui| {
            for index in 0..self.docs.len() {
                let name = self.docs[index].name.clone();
                if ui.selectable_label(index == self.active, name).clicked() {
                    self.active = index;
                }
            }
        });

        ui.collapsing("FOLDERS", |ui| {
            let folders = self.settings.workspace_folders.clone();
            for folder in folders {
                self.folder_row(ui, &folder, 0);
            }
            if self.settings.workspace_folders.is_empty() {
                ui.label(egui::RichText::new("No folder open").weak());
                if ui.small_button("Open Folder...").clicked() {
                    self.open_folder();
                }
            }
        });

        ui.collapsing("RECENT", |ui| {
            let recent = self.settings.recent_files.clone();
            for path in recent {
                let name = PathBuf::from(&path)
                    .file_name()
                    .map(|value| value.to_string_lossy().into_owned())
                    .unwrap_or(path.clone());
                if ui.button(name).clicked() {
                    self.open_path(PathBuf::from(path));
                }
            }
        });
    }

    fn folder_row(&mut self, ui: &mut egui::Ui, path: &str, depth: usize) {
        let pathbuf = PathBuf::from(path);
        let name = pathbuf
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string());
        let is_dir = pathbuf.is_dir();
        let expanded = self.expanded.contains(path);
        let indent = (depth * 14) as f32;
        let accent = hex_color(&self.theme().accent);
        ui.horizontal(|ui| {
            ui.add_space(indent);
            if is_dir {
                if self.icon_button(
                    ui,
                    if expanded { "chevron-down" } else { "chevron-right" },
                    12.0,
                    accent,
                ) {
                    if expanded {
                        self.expanded.remove(path);
                    } else {
                        self.expanded.insert(path.to_string());
                    }
                }
                self.icon(ui, "folder", 14.0, accent);
            } else {
                ui.add_space(16.0);
                self.icon(ui, "file", 14.0, accent);
            }
            if ui.button(name).clicked() {
                if is_dir {
                    if expanded {
                        self.expanded.remove(path);
                    } else {
                        self.expanded.insert(path.to_string());
                    }
                } else {
                    self.open_path(pathbuf.clone());
                }
            }
        });
        if is_dir && expanded {
            if let Ok(entries) = std::fs::read_dir(&pathbuf) {
                let mut children: Vec<String> = entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path().to_string_lossy().into_owned())
                    .collect();
                children.sort();
                for child in children {
                    self.folder_row(ui, &child, depth + 1);
                }
            }
        }
    }

    fn ui_search(&mut self, ui: &mut egui::Ui) {
        let accent = hex_color(&self.theme().accent);
        ui.horizontal(|ui| {
            self.icon(ui, "search", 15.0, accent);
            ui.heading("Find & Replace");
        });
        ui.text_edit_singleline(&mut self.find_text);
        ui.horizontal(|ui| {
            self.icon(ui, "refresh-cw", 14.0, accent);
            ui.text_edit_singleline(&mut self.replace_text);
        });
        if ui.checkbox(&mut self.folder_search, "Search all workspace files").changed() {
            self.folder_hits.clear();
        }
        ui.horizontal(|ui| {
            if ui.button("Find All").clicked() { self.do_find(); }
            if ui.button("Replace All").clicked() { self.do_replace(); }
        });
        ui.separator();
        if self.find_text.is_empty() {
            ui.label(egui::RichText::new("Enter text to search").weak());
            return;
        }

        if self.folder_search {
            ui.label(egui::RichText::new(format!("Workspace: {} matches", self.folder_hits.len())).strong());
            for hit in self.folder_hits.clone() {
                let name = hit
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| hit.path.to_string_lossy().into_owned());
                if ui.button(format!("{}:{} — {}", name, hit.line, hit.text)).clicked() {
                    let path = hit.path.clone();
                    let char_index = hit.char_index;
                    self.open_path(path);
                    let content = self.docs[self.active].content.clone();
                    self.cursor = char_to_line_col(&content, char_index);
                    self.jump_line = Some(hit.line);
                }
            }
            return;
        }

        let results = self.docs[self.active].find_all(&self.find_text);
        ui.label(format!("{} matches in this file", results.len()));
        let content = self.docs[self.active].content.clone();
        for (start, _end) in results.iter().take(100) {
            let (line, _) = char_to_line_col(&content, *start);
            let text = content.lines().nth(line - 1).unwrap_or("").trim();
            if ui.button(format!("{}: {}", line, text)).clicked() {
                self.cursor = (line, 1);
                self.jump_line = Some(line);
            }
        }
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui) {
        let accent = hex_color(&self.theme().accent);
        ui.horizontal(|ui| {
            self.icon(ui, "settings", 16.0, accent);
            ui.heading("Settings");
        });
        ui.separator();
        match self.settings_section {
            1 => {
                ui.heading("Editor");
                ui.checkbox(&mut self.settings.show_line_numbers, "Show line numbers");
                ui.checkbox(&mut self.settings.word_wrap, "Word wrap");
                ui.checkbox(&mut self.settings.insert_spaces, "Insert spaces for Tab");
                if ui.add(egui::Slider::new(&mut self.settings.font_size, 8.0..=32.0).text("Font size")).changed() {
                    self.settings_dirty = true;
                }
                if ui.add(egui::Slider::new(&mut self.settings.tab_size, 2..=8).text("Tab size")).changed() {
                    self.settings_dirty = true;
                }
            }
            2 => {
                ui.heading("Appearance");
                self.ui_theme_picker(ui);
                if ui.checkbox(&mut self.settings.follow_system_theme, "Auto theme follows system light/dark").clicked() {
                    self.settings_dirty = true;
                }
                if ui.checkbox(&mut self.settings.animate_themes, "Animate theme changes").clicked() {
                    self.settings_dirty = true;
                }
                if ui.checkbox(&mut self.settings.show_minimap, "Show minimap").clicked() {
                    self.settings_dirty = true;
                }
                if ui.checkbox(&mut self.settings.show_toasts, "Show notifications").clicked() {
                    self.settings_dirty = true;
                }
                ui.separator();
                self.ui_layout_settings(ui);
            }
            3 => {
                ui.heading("Custom themes");
                self.ui_custom_themes(ui);
            }
             4 => {
                ui.heading("Workspace");
                self.ui_layout_settings(ui);
                self.ui_layout_sizes(ui);
                if ui.add(egui::Slider::new(&mut self.settings.autosave_secs, 1..=60).text("Autosave (seconds)")).changed() {
                    self.settings_dirty = true;
                }
                if ui.button("Open Folder...").clicked() {
                    self.open_folder();
                }
            }
            5 => {
                ui.heading("AI Assistant");
                ui.label(egui::RichText::new("Works with any OpenAI-compatible chat endpoint.").weak());
                ui.label("Endpoint");
                ui.text_edit_singleline(&mut self.settings.ai_endpoint);
                ui.label("Model");
                ui.text_edit_singleline(&mut self.settings.ai_model);
                ui.horizontal(|ui| {
                    ui.label("API key");
                    ui.checkbox(&mut self.show_ai_key, "show");
                });
                egui::TextEdit::singleline(&mut self.settings.ai_key)
                    .password(!self.show_ai_key)
                    .hint_text("sk-...")
                    .show(ui);
                ui.label(egui::RichText::new("Your key is stored locally in settings.json.").weak());
                if ui.button("Open Assistant").clicked() {
                    self.panel_view = PanelView::Ai;
                    self.settings.panel_visible = true;
                }
                self.settings_dirty = true;
            }
            6 => {
                ui.heading("Notes");
                ui.label("Notes folder");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.settings.notes_folder);
                    if ui.button("Choose...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.settings.notes_folder = path.to_string_lossy().into_owned();
                        }
                    }
                });
                if ui.button("Create today's note").clicked() {
                    self.new_daily_note();
                }
                if ui.button("Open quick capture").clicked() {
                    self.quick_open = true;
                }
                ui.label(egui::RichText::new("Quick capture and daily notes are Markdown files.").weak());
                self.settings_dirty = true;
            }
            _ => {
                ui.heading("General");
                self.ui_theme_picker(ui);
                ui.separator();
                ui.checkbox(&mut self.settings.show_line_numbers, "Show line numbers");
                ui.checkbox(&mut self.settings.word_wrap, "Word wrap");
                if ui.add(egui::Slider::new(&mut self.settings.font_size, 8.0..=32.0).text("Font size")).changed() {
                    self.settings_dirty = true;
                }
                if ui.add(egui::Slider::new(&mut self.settings.autosave_secs, 1..=60).text("Autosave (seconds)")).changed() {
                    self.settings_dirty = true;
                }
            }
        }
    }

    fn ui_layout_sizes(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.label(egui::RichText::new("Drag the dividers between panels, or use these controls.").weak());
        if ui.add(egui::Slider::new(&mut self.settings.side_width, 180.0..=560.0).text("Side bar width")).changed() {
            self.settings_dirty = true;
        }
        if ui.add(egui::Slider::new(&mut self.settings.panel_size, 120.0..=640.0).text("Panel size")).changed() {
            self.settings_dirty = true;
        }
        if ui.add(egui::Slider::new(&mut self.settings.minimap_width, 36.0..=120.0).text("Minimap width")).changed() {
            self.settings_dirty = true;
        }
        ui.horizontal(|ui| {
            ui.label("Side bar:");
            if ui.selectable_label(self.settings.side_side == "left", "Left").clicked() {
                self.settings.side_side = "left".into();
                self.settings_dirty = true;
            }
            if ui.selectable_label(self.settings.side_side == "right", "Right").clicked() {
                self.settings.side_side = "right".into();
                self.settings_dirty = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Panel:");
            for option in ["bottom", "left", "right"] {
                if ui.selectable_label(self.settings.panel_position == option, option).clicked() {
                    self.settings.panel_position = option.into();
                    self.settings_dirty = true;
                }
            }
        });
        if ui.button("Reset layout sizes").clicked() {
            self.settings.side_width = 290.0;
            self.settings.panel_size = 220.0;
            self.settings.minimap_width = 56.0;
            self.settings_dirty = true;
        }
    }

    fn ui_theme_picker(&mut self, ui: &mut egui::Ui) {
        let themes = self.all_themes();
        let current = self.target_theme();
        egui::ComboBox::from_label("Theme")
            .selected_text(current.name.clone())
            .show_ui(ui, |ui| {
                for theme in &themes {
                    if ui
                        .selectable_label(theme.id == self.settings.theme_id, &theme.name)
                        .clicked()
                    {
                        self.settings.theme_id = theme.id.clone();
                        self.settings_dirty = true;
                    }
                }
            });
    }

    fn ui_layout_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Workspace layout");
        egui::ComboBox::from_label("Side bar")
            .selected_text(self.settings.side_side.clone())
            .show_ui(ui, |ui| {
                if ui.selectable_label(self.settings.side_side == "left", "Left").clicked() {
                    self.settings.side_side = "left".into();
                    self.settings_dirty = true;
                }
                if ui.selectable_label(self.settings.side_side == "right", "Right").clicked() {
                    self.settings.side_side = "right".into();
                    self.settings_dirty = true;
                }
            });
        egui::ComboBox::from_label("Panel")
            .selected_text(self.settings.panel_position.clone())
            .show_ui(ui, |ui| {
                for option in ["bottom", "right", "left"] {
                    if ui.selectable_label(self.settings.panel_position == option, option).clicked() {
                        self.settings.panel_position = option.into();
                        self.settings_dirty = true;
                    }
                }
            });
    }

    fn ui_custom_themes(&mut self, ui: &mut egui::Ui) {
        if ui.button("Create theme from current").clicked() {
            let mut theme = self.theme();
            let id = format!(
                "custom-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|value| value.as_millis())
                    .unwrap_or(0)
            );
            theme.id = id.clone();
            theme.name = "My Theme".to_string();
            self.settings.custom_themes.push(theme);
            self.settings.theme_id = id.clone();
            self.editing_theme = Some(id);
            self.settings_dirty = true;
        }
        let custom = self.settings.custom_themes.clone();
        for theme in custom {
            ui.horizontal(|ui| {
                if ui.button(&theme.name).clicked() {
                    self.settings.theme_id = theme.id.clone();
                    self.editing_theme = Some(theme.id.clone());
                    self.settings_dirty = true;
                }
                if ui.small_button("Delete").clicked() {
                    self.settings.custom_themes.retain(|item| item.id != theme.id);
                    if self.settings.theme_id == theme.id {
                        self.settings.theme_id = "dark+".into();
                    }
                    if self.editing_theme.as_deref() == Some(theme.id.as_str()) {
                        self.editing_theme = None;
                    }
                    self.settings_dirty = true;
                }
            });
        }
        if let Some(id) = self.editing_theme.clone() {
            if let Some(index) = self.settings.custom_themes.iter().position(|theme| theme.id == id) {
                ui.separator();
                ui.label("Editing theme");
                let theme = &mut self.settings.custom_themes[index];
                ui.text_edit_singleline(&mut theme.name);
                self.theme_color(ui, index, "Background", |t| &mut t.bg);
                self.theme_color(ui, index, "Text", |t| &mut t.text);
                self.theme_color(ui, index, "Chrome", |t| &mut t.chrome);
                self.theme_color(ui, index, "Chrome text", |t| &mut t.chrome_text);
                self.theme_color(ui, index, "Accent", |t| &mut t.accent);
                self.theme_color(ui, index, "Selection", |t| &mut t.selection);
                self.theme_color(ui, index, "Line highlight", |t| &mut t.line_bg);
                self.theme_color(ui, index, "Muted", |t| &mut t.muted);
                self.settings_dirty = true;
            }
        }
    }

    fn theme_color(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        label: &str,
        field: fn(&mut Theme) -> &mut String,
    ) {
        let value = field(&mut self.settings.custom_themes[index]).clone();
        let color = hex_color(&value);
        let mut rgb = [color.r(), color.g(), color.b()];
        ui.horizontal(|ui| {
            ui.label(label);
            if ui.color_edit_button_srgb(&mut rgb).changed() {
                *field(&mut self.settings.custom_themes[index]) =
                    color_to_hex(egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
                self.settings_dirty = true;
            }
        });
    }

    fn ui_panel(&mut self, ui: &mut egui::Ui) {
        if !self.settings.panel_visible {
            return;
        }
        let chrome = hex_color(&self.theme().chrome);
        let border = hex_color(&self.theme().border);
        let frame = || {
            egui::Frame::new()
                .fill(chrome)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .stroke(egui::Stroke::new(1.0, border))
        };
        let build = |ui: &mut egui::Ui, app: &mut Self| {
            ui.horizontal(|ui| {
                if ui.selectable_label(app.panel_view == PanelView::Problems, "Problems").clicked() {
                    app.panel_view = PanelView::Problems;
                }
                if ui.selectable_label(app.panel_view == PanelView::Output, "Output").clicked() {
                    app.panel_view = PanelView::Output;
                }
                if ui.selectable_label(app.panel_view == PanelView::Ai, "AI Assistant").clicked() {
                    app.panel_view = PanelView::Ai;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some((dx, dy)) = app.dock_grip(ui, "panel") {
                        let position = if dy > 60.0 {
                            "bottom"
                        } else if dx < -60.0 {
                            "left"
                        } else if dx > 60.0 {
                            "right"
                        } else {
                            app.settings.panel_position.as_str()
                        };
                        app.settings.panel_position = position.to_string();
                        app.settings_dirty = true;
                    }
                });
            });
            ui.separator();
            match app.panel_view {
                PanelView::Problems => {
                    let modified = app.docs.iter().filter(|doc| doc.modified).count();
                    if modified == 0 {
                        ui.label("No problems detected");
                    } else {
                        for doc in &app.docs {
                            if doc.modified {
                                ui.label(format!("Unsaved changes: {}", doc.name));
                            }
                        }
                    }
                }
                PanelView::Output => {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for line in &app.output {
                            ui.label(line);
                        }
                    });
                }
                PanelView::Ai => app.ui_ai(ui),
            }
        };
        let size = self.settings.panel_size;
        match self.settings.panel_position.as_str() {
            "right" => {
                egui::Panel::right("panel_right")
                    .exact_size(size)
                    .frame(frame())
                    .show(ui, |ui| build(ui, self));
            }
            "left" => {
                egui::Panel::left("panel_left")
                    .exact_size(size)
                    .frame(frame())
                    .show(ui, |ui| build(ui, self));
            }
            _ => {
                egui::Panel::bottom("panel_bottom")
                    .exact_size(size)
                    .frame(frame())
                    .show(ui, |ui| build(ui, self));
            }
        }
    }

    fn ui_ai(&mut self, ui: &mut egui::Ui) {
        let accent = hex_color(&self.current_theme.accent);
        ui.horizontal(|ui| {
            if ui.button("Explain").clicked() {
                self.ask_ai("Explain the supplied text clearly and briefly.");
            }
            if ui.button("Improve").clicked() {
                self.ask_ai("Rewrite the supplied text to be clearer and more concise. Return only the improved text.");
            }
            if ui.button("Checklist").clicked() {
                self.ask_ai("Turn the supplied notes into a short actionable checklist. Use Markdown checkboxes.");
            }
            if ui.button("Summarize").clicked() {
                self.ask_ai("Summarize the supplied text into concise bullet points.");
            }
        });
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.ai_prompt)
                    .hint_text("Optional instruction for the current note or selection...")
                    .desired_rows(2)
                    .desired_width(f32::INFINITY),
            );
        });
        ui.horizontal(|ui| {
            let label = if self.ai_busy { "Thinking..." } else { "Ask AI" };
            if ui.button(label).clicked() {
                let instruction = if self.ai_prompt.trim().is_empty() {
                    "Respond helpfully to the supplied note."
                } else {
                    self.ai_prompt.trim()
                }
                .to_string();
                self.ask_ai(&instruction);
            }
            ui.label(
                egui::RichText::new(if self.selection.is_empty() {
                    "Using whole document"
                } else {
                    "Using selected text"
                })
                .weak()
                .size(11.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Copy").clicked() {
                    ui.ctx().copy_text(self.ai_response.clone());
                    self.toast("Response copied", ToastKind::Success);
                }
            });
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.ai_response.is_empty() {
                ui.label(
                    egui::RichText::new("Ask a question or choose an action above. Select text in the editor to focus the AI on it.")
                        .weak(),
                );
            } else {
                ui.label(egui::RichText::new(&self.ai_response).color(accent));
            }
        });
    }

    fn ui_center(&mut self, ui: &mut egui::Ui) {
        let show_numbers = self.settings.show_line_numbers;
        let font_size = self.settings.font_size;
        let word_wrap = self.settings.word_wrap;
        let show_minimap = self.settings.show_minimap;
        let minimap_width = self.settings.minimap_width.max(36.0);
        let bg = hex_color(&self.current_theme.bg);
        let jump = self
            .jump_line
            .take()
            .map(|line| (line.saturating_sub(1)) as f32 * font_size * 1.55);

        let area = ui.available_rect_before_wrap();
        let editor_width = if show_minimap {
            (area.width() - minimap_width - 6.0).max(120.0)
        } else {
            area.width()
        };
        let editor_rect = egui::Rect::from_min_size(
            area.min,
            egui::vec2(editor_width, area.height()),
        );

        let longest = self.docs[self.active]
            .content
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(80);
        let wide_width = (longest as f32 * font_size * 0.62 + 80.0).clamp(400.0, 12000.0);

        let mut new_cursor = None;
        let mut changed = false;
        let mut selection = String::new();

        let mut editor_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(editor_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        let mut scroll = if word_wrap {
            egui::ScrollArea::vertical()
        } else {
            egui::ScrollArea::both()
        };
        if let Some(offset) = jump {
            scroll = scroll.vertical_scroll_offset(offset);
        }
        scroll.show(&mut editor_ui, |ui| {
            ui.horizontal(|ui| {
                if !word_wrap {
                    ui.set_width(wide_width);
                }
                let doc = &mut self.docs[self.active];
                if show_numbers {
                    ui.vertical(|ui| {
                        for line in 1..=doc.line_count() {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(format!("{:>4}", line))
                                        .size(font_size * 0.9)
                                        .weak(),
                                )
                                .sense(egui::Sense::hover()),
                            );
                        }
                    });
                }
                let editor = egui::TextEdit::multiline(&mut doc.content)
                    .code_editor()
                    .frame(
                        egui::Frame::new()
                            .fill(bg)
                            .inner_margin(egui::Margin::symmetric(10, 8)),
                    )
                    .desired_width(f32::INFINITY);
                let output = editor.show(ui);
                changed = output.response.changed();
                if let Some(range) = output.cursor_range.clone() {
                    new_cursor = Some(char_to_line_col(&doc.content, range.primary.index.0));
                    let span = range.as_sorted_char_range();
                    if span.end.0 > span.start.0 {
                        selection = char_range_text(&doc.content, span.start.0, span.end.0);
                    }
                }
            });
        });

        if show_minimap {
            let minimap_rect = egui::Rect::from_min_size(
                egui::pos2(editor_rect.right() + 6.0, area.top()),
                egui::vec2(minimap_width - 6.0, area.height()),
            );
            self.ui_minimap(ui, minimap_rect);
        }

        if changed {
            self.docs[self.active].modified = true;
        }
        if let Some(cursor) = new_cursor {
            self.cursor = cursor;
        }
        self.selection = selection;
    }

    fn ui_minimap(&mut self, ui: &mut egui::Ui, area: egui::Rect) {
        let lines: Vec<String> = self.docs[self.active]
            .content
            .lines()
            .map(str::to_string)
            .collect();
        let count = lines.len().max(1);
        let longest = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let row_height = (area.height() / count as f32).max(1.0);
        let width = area.width();
        let accent = hex_color(&self.current_theme.accent);
        let muted = hex_color(&self.current_theme.muted);
        let background = ui.visuals().faint_bg_color;

        ui.painter().rect_filled(area, 3.0, background);
        for (index, line) in lines.iter().enumerate() {
            let y = area.top() + index as f32 * row_height;
            let bar_width = (line.chars().count() as f32 / longest as f32) * (width - 4.0);
            if index + 1 == self.cursor.0 {
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(area.left(), y),
                        egui::vec2(width, row_height),
                    ),
                    0.0,
                    egui::Color32::from_white_alpha(22),
                );
            }
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(area.left() + 2.0, y + 0.5),
                    egui::vec2(bar_width.max(1.0), row_height.max(1.0) - 1.0),
                ),
                1.0,
                if index + 1 == self.cursor.0 {
                    accent
                } else {
                    muted
                },
            );
        }
        let response = ui.interact(area, ui.id().with("minimap"), egui::Sense::click_and_drag());
        if let Some(position) = response.interact_pointer_pos() {
            let line =
                (((position.y - area.top()) / area.height()) * count as f32).floor() as usize + 1;
            let line = line.min(count);
            self.jump_line = Some(line);
            self.cursor = (line, 1);
        }
    }

    fn ui_status(&mut self, ui: &mut egui::Ui) {
        let accent = hex_color(&self.theme().accent);
        let white = |text: String| egui::RichText::new(text).color(egui::Color32::WHITE).size(12.0);
        let theme_name = self.theme().name;
        let tab_size = self.settings.tab_size;
        let content = &self.docs[self.active].content;
        let words = content.split_whitespace().count();
        let chars = content.chars().count();
        let lines = self.docs[self.active].line_count();
        let reading = if words == 0 { 0 } else { (words / 200).max(1) };
        egui::Panel::bottom("status_bar")
            .frame(
                egui::Frame::new()
                    .fill(accent)
                    .inner_margin(egui::Margin::symmetric(10, 4)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(white(format!("Ln {}, Col {}", self.cursor.0, self.cursor.1)));
                    ui.label(white("|".to_string()));
                    let modified = self.docs[self.active].modified;
                    ui.label(white(if modified { "Modified" } else { "Saved" }.to_string()));
                    ui.label(white("|".to_string()));
                    ui.label(white(format!("Spaces: {}", tab_size)));
                    ui.label(white("|".to_string()));
                    ui.label(white(format!("{} words · {} chars · {} lines", words, chars, lines)));
                    ui.label(white("|".to_string()));
                    ui.label(white(format!("~{} min read", reading)));
                    ui.label(white("|".to_string()));
                    ui.label(white("UTF-8".to_string()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("-").clicked() {
                            self.settings.font_size = (self.settings.font_size - 1.0).max(8.0);
                            self.settings_dirty = true;
                        }
                        ui.label(white(format!("{}%", (self.settings.font_size / 14.0 * 100.0) as i32)));
                        if ui.small_button("+").clicked() {
                            self.settings.font_size = (self.settings.font_size + 1.0).min(32.0);
                            self.settings_dirty = true;
                        }
                        let wrap_label = if self.settings.word_wrap { "Wrap: On" } else { "Wrap: Off" };
                        if ui.button(wrap_label).clicked() {
                            self.settings.word_wrap = !self.settings.word_wrap;
                            self.settings_dirty = true;
                        }
                        ui.label(white(theme_name));
                    });
                });
            });
    }

    fn ui_palette(&mut self, ctx: &egui::Context) {
        if !self.palette_open {
            return;
        }
        let mut open = self.palette_open;
        let mut close = false;
        let mut execute: Option<Command> = None;
        let query = self.palette_query.to_lowercase();
        let filtered: Vec<(String, String, Command)> = self
            .commands()
            .into_iter()
            .filter(|(label, hint, _)| {
                label.to_lowercase().contains(&query) || hint.to_lowercase().contains(&query)
            })
            .collect();
        if self.palette_selected >= filtered.len() {
            self.palette_selected = 0;
        }

        egui::Window::new("Command Palette")
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -40.0))
            .fixed_size(egui::vec2(540.0, 440.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                let input = egui::TextEdit::singleline(&mut self.palette_query)
                    .hint_text("Search commands, files, and actions...")
                    .show(ui);
                input.response.request_focus();
                ui.separator();
                egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                    if filtered.is_empty() {
                        ui.label(egui::RichText::new("No matching commands").weak());
                    }
                    for (index, (label, hint, command)) in filtered.iter().enumerate() {
                        let selected = index == self.palette_selected;
                        let response = ui.horizontal(|ui| {
                            let click = ui.selectable_label(selected, egui::RichText::new(label));
                            ui.label(egui::RichText::new(hint).weak().size(11.0));
                            click
                        });
                        if response.inner.clicked() {
                            execute = Some(*command);
                        }
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    self.palette_selected = (self.palette_selected + 1).min(filtered.len().saturating_sub(1));
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    self.palette_selected = self.palette_selected.saturating_sub(1);
                }
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some((_, _, command)) = filtered.get(self.palette_selected) {
                        execute = Some(*command);
                    }
                }
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    close = true;
                }
            });

        if close {
            open = false;
        }
        self.palette_open = open;
        if let Some(command) = execute {
            self.palette_open = false;
            self.palette_query.clear();
            self.palette_selected = 0;
            self.run_command(ctx, command);
        }
    }

    fn ui_quick(&mut self, ctx: &egui::Context) {
        if !self.quick_open {
            return;
        }
        let mut open = self.quick_open;
        let mut save = false;
        let mut cancel = false;
        egui::Window::new("Quick capture")
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(480.0, 260.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Drop a thought — it is timestamped and added to quick-capture.md")
                        .weak(),
                );
                let input = egui::TextEdit::multiline(&mut self.quick_text)
                    .hint_text("What's on your mind?")
                    .desired_rows(5)
                    .desired_width(f32::INFINITY)
                    .show(ui);
                input.response.request_focus();
                ui.horizontal(|ui| {
                    if ui.button("Save note").clicked() { save = true; }
                    if ui.button("Cancel").clicked() { cancel = true; }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(tools::now_time_string()).weak());
                    });
                });
            });
        if cancel {
            open = false;
        }
        self.quick_open = open;
        if save {
            self.save_quick_capture();
        }
    }

    fn ui_toasts(&mut self, ctx: &egui::Context) {
        self.toasts
            .retain(|toast| toast.created.elapsed() < Duration::from_secs(4));
        for (index, toast) in self.toasts.iter().enumerate() {
            let age = toast.created.elapsed().as_secs_f32();
            let opacity = if age < 0.2 {
                age / 0.2
            } else if age > 3.5 {
                (4.0 - age) / 0.5
            } else {
                1.0
            }
            .clamp(0.0, 1.0);
            let base = match toast.kind {
                ToastKind::Info => hex_color(&self.current_theme.accent),
                ToastKind::Success => egui::Color32::from_rgb(35, 140, 90),
                ToastKind::Error => egui::Color32::from_rgb(180, 55, 55),
            };
            let color = egui::Color32::from_rgba_premultiplied(
                (base.r() as f32 * opacity) as u8,
                (base.g() as f32 * opacity) as u8,
                (base.b() as f32 * opacity) as u8,
                (230.0 * opacity) as u8,
            );
            egui::Window::new(format!("toast-{}", index))
                .anchor(
                    egui::Align2::RIGHT_TOP,
                    egui::vec2(-14.0, 68.0 + index as f32 * 52.0),
                )
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .frame(
                    egui::Frame::new()
                        .fill(color)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .corner_radius(egui::CornerRadius::same(6)),
                )
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(&toast.message).color(egui::Color32::WHITE));
                });
        }
        if !self.toasts.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(120));
        }
    }

    fn ui_about(&mut self, ctx: &egui::Context) {
        if !self.about_open {
            return;
        }
        let mut open = self.about_open;
        egui::Window::new("About Pezli Editor")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Pezli Editor v1.0.0");
                ui.label("A native desktop text editor built with Rust and egui.");
                ui.label("Features: tabs, themes, folders, find and replace, command palette, minimap, workspace search, AI assistant, daily notes, quick capture, and Markdown preview.");
                ui.label("Built by Beauttah Kipruto, MUST.");
            });
        self.about_open = open;
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    let command = modifiers.ctrl || modifiers.command;
                    if command {
                        match key {
                            egui::Key::N => self.new_doc(),
                            egui::Key::O => self.open_file(),
                            egui::Key::S => self.save_file(modifiers.shift),
                            egui::Key::F if modifiers.shift => {
                                self.side_view = SideView::Search;
                                self.folder_search = true;
                                self.settings.side_visible = true;
                            }
                            egui::Key::F => {
                                self.side_view = SideView::Search;
                                self.settings.side_visible = true;
                            }
                            egui::Key::P => self.palette_open = true,
                            egui::Key::T => self.next_theme(),
                            egui::Key::B => {
                                self.settings.side_visible = !self.settings.side_visible;
                                self.settings_dirty = true;
                            }
                            egui::Key::Q if modifiers.shift => self.quick_open = true,
                            egui::Key::D if modifiers.shift => self.new_daily_note(),
                            egui::Key::M if modifiers.shift => {
                                self.side_view = if self.side_view == SideView::Preview {
                                    SideView::Explorer
                                } else {
                                    SideView::Preview
                                };
                                self.settings.side_visible = true;
                            }
                            egui::Key::A if modifiers.alt => {
                                self.panel_view = PanelView::Ai;
                                self.settings.panel_visible = true;
                            }
                            egui::Key::Equals | egui::Key::Plus => {
                                self.settings.font_size = (self.settings.font_size + 1.0).min(32.0);
                                self.settings_dirty = true;
                            }
                            egui::Key::Minus => {
                                self.settings.font_size = (self.settings.font_size - 1.0).max(8.0);
                                self.settings_dirty = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }
}

impl Drop for PezliApp {
    fn drop(&mut self) {
        self.capture_session();
        self.settings.save();
    }
}

impl eframe::App for PezliApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.settings.follow_system_theme
            && self.last_system_check.elapsed() >= Duration::from_secs(60)
        {
            self.system_dark = tools::system_prefers_dark();
            self.last_system_check = Instant::now();
        }
        self.apply_style(&ctx);
        self.poll_ai();
        self.shortcuts(&ctx);

        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect()
        });
        for path in dropped {
            if path.is_dir() {
                let text = path.to_string_lossy().into_owned();
                if !self.settings.workspace_folders.contains(&text) {
                    self.settings.workspace_folders.push(text);
                    self.settings_dirty = true;
                }
                self.toast(format!("Workspace: {}", path.display()), ToastKind::Success);
            } else {
                self.open_path(path);
            }
        }

        self.ui_menu(ui);
        self.ui_toolbar(ui);
        self.ui_tabs(ui);
        self.ui_side(ui);
        self.ui_panel(ui);
        self.ui_status(ui);
        let central_rect = ui.available_rect_before_wrap();
        egui::CentralPanel::default().show(ui, |ui| {
            if self.side_view == SideView::Settings {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_max_width(820.0);
                    ui.add_space(12.0);
                    self.ui_settings(ui);
                });
            } else {
                self.ui_center(ui);
            }
        });
        self.ui_layout_splitters(ui, central_rect);

        self.ui_palette(&ctx);
        self.ui_quick(&ctx);
        self.ui_about(&ctx);
        self.ui_toasts(&ctx);

        if self.settings_dirty {
            self.settings.save();
            self.settings_dirty = false;
        }
        if self.last_autosave.elapsed() >= Duration::from_secs(self.settings.autosave_secs.max(1)) {
            self.capture_session();
            self.settings.save();
            self.last_autosave = Instant::now();
        }
    }
}
