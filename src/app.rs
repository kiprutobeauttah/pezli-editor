use crate::editor::{char_to_line_col, Document};
use crate::settings::Settings;
use crate::theme::{builtin_themes, color_to_hex, hex_color, Theme};
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SideView {
    Explorer,
    Search,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PanelView {
    Problems,
    Output,
}

pub struct PezliApp {
    docs: Vec<Document>,
    active: usize,
    settings: Settings,
    side_view: SideView,
    panel_view: PanelView,
    find_text: String,
    replace_text: String,
    output: Vec<String>,
    cursor: (usize, usize),
    expanded: HashSet<String>,
    settings_dirty: bool,
    about_open: bool,
    editing_theme: Option<String>,
}

impl PezliApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        Self {
            docs: vec![Document::untitled("Untitled-1".to_string())],
            active: 0,
            settings,
            side_view: SideView::Explorer,
            panel_view: PanelView::Problems,
            find_text: String::new(),
            replace_text: String::new(),
            output: vec!["Pezli Editor ready".to_string()],
            cursor: (1, 1),
            expanded: HashSet::new(),
            settings_dirty: false,
            about_open: false,
            editing_theme: None,
        }
    }

    fn theme(&self) -> Theme {
        let mut themes = builtin_themes();
        themes.extend(self.settings.custom_themes.clone());
        themes
            .into_iter()
            .find(|theme| theme.id == self.settings.theme_id)
            .unwrap_or_else(|| builtin_themes()[0].clone())
    }

    fn all_themes(&self) -> Vec<Theme> {
        let mut themes = builtin_themes();
        themes.extend(self.settings.custom_themes.clone());
        themes
    }

    fn log(&mut self, message: impl Into<String>) {
        self.output.push(message.into());
        if self.output.len() > 200 {
            self.output.remove(0);
        }
    }

    fn apply_style(&self, ctx: &egui::Context) {
        let theme = self.theme();
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = hex_color(&theme.chrome);
        visuals.panel_fill = hex_color(&theme.bg);
        visuals.text_color = hex_color(&theme.text);
        visuals.widgets.noninteractive.fg_stroke.color = hex_color(&theme.muted);
        visuals.widgets.inactive.fg_stroke.color = hex_color(&theme.chrome_text);
        visuals.widgets.hovered.fg_stroke.color = hex_color(&theme.chrome_text);
        visuals.widgets.active.fg_stroke.color = hex_color(&theme.chrome_text);
        visuals.selection.bg_fill = hex_color(&theme.selection);
        visuals.selection.stroke.color = hex_color(&theme.accent);
        visuals.hyperlink_color = hex_color(&theme.accent);
        ctx.set_visuals(visuals);
        let font_size = self.settings.font_size;
        ctx.all_styles_mut(|style| {
            for font in style.text_styles.values_mut() {
                font.size = font_size;
            }
        });
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
                let text = path.to_string_lossy().into_owned();
                self.settings.recent_files.retain(|item| item != &text);
                self.settings.recent_files.insert(0, text);
                self.settings.recent_files.truncate(12);
                self.settings_dirty = true;
            }
            Err(error) => self.log(format!("Failed to open file: {}", error)),
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
            }
            Err(error) => self.log(format!("Failed to save: {}", error)),
        }
    }

    fn do_find(&mut self) {
        let results = self.docs[self.active].find_all(&self.find_text);
        self.log(format!("Found {} matches", results.len()));
    }

    fn do_replace(&mut self) {
        let count = self.docs[self.active].replace_all(&self.find_text, &self.replace_text);
        self.log(format!("Replaced {} occurrences", count));
    }

    fn ui_menu(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New File").clicked() { self.new_doc(); ui.close_menu(); }
                    if ui.button("Open File...").clicked() { self.open_file(); ui.close_menu(); }
                    if ui.button("Open Folder...").clicked() { self.open_folder(); ui.close_menu(); }
                    ui.separator();
                    if ui.button("Save").clicked() { self.save_file(false); ui.close_menu(); }
                    if ui.button("Save As...").clicked() { self.save_file(true); ui.close_menu(); }
                    ui.separator();
                    if ui.button("Exit").clicked() { ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close); }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Find...").clicked() {
                        self.side_view = SideView::Search;
                        self.settings.side_visible = true;
                        self.settings_dirty = true;
                        ui.close_menu();
                    }
                    if ui.button("Replace All").clicked() { self.do_replace(); ui.close_menu(); }
                });
                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.settings.side_visible, "Side Bar").clicked() { self.settings_dirty = true; }
                    if ui.checkbox(&mut self.settings.panel_visible, "Panel").clicked() { self.settings_dirty = true; }
                    ui.separator();
                    if ui.button("Switch Theme").clicked() {
                        let themes = self.all_themes();
                        let index = themes.iter().position(|t| t.id == self.settings.theme_id).unwrap_or(0);
                        self.settings.theme_id = themes[(index + 1) % themes.len()].id.clone();
                        self.settings_dirty = true;
                        ui.close_menu();
                    }
                    if ui.button("Settings").clicked() {
                        self.side_view = SideView::Settings;
                        self.settings.side_visible = true;
                        self.settings_dirty = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("Keyboard Shortcuts").clicked() {
                        self.log("Shortcuts: Ctrl+N new, Ctrl+O open, Ctrl+S save, Ctrl+Shift+S save as, Ctrl+F find");
                        ui.close_menu();
                    }
                    if ui.button("About").clicked() { self.about_open = true; ui.close_menu(); }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(self.theme().name).weak());
                });
            });
        });
    }

    fn ui_tabs(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("tabs_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                for index in 0..self.docs.len() {
                    let name = if self.docs[index].modified {
                        format!("{} •", self.docs[index].name)
                    } else {
                        self.docs[index].name.clone()
                    };
                    let selected = index == self.active;
                    if ui.selectable_label(selected, name).clicked() {
                        self.active = index;
                    }
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
                if ui.selectable_label(app.side_view == SideView::Explorer, "Explorer").clicked() {
                    app.side_view = SideView::Explorer;
                }
                if ui.selectable_label(app.side_view == SideView::Search, "Search").clicked() {
                    app.side_view = SideView::Search;
                }
                if ui.selectable_label(app.side_view == SideView::Settings, "Settings").clicked() {
                    app.side_view = SideView::Settings;
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| match app.side_view {
                SideView::Explorer => app.ui_explorer(ui),
                SideView::Search => app.ui_search(ui),
                SideView::Settings => app.ui_settings(ui),
            });
        };

        if side_right {
            egui::Panel::right("side_panel").default_size(270.0).show(ui, |ui| build(ui, self));
        } else {
            egui::Panel::left("side_panel").default_size(270.0).show(ui, |ui| build(ui, self));
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
        ui.horizontal(|ui| {
            ui.add_space(indent);
            if is_dir {
                if ui.button(if expanded { "▾" } else { "▸" }).clicked() {
                    if expanded {
                        self.expanded.remove(path);
                    } else {
                        self.expanded.insert(path.to_string());
                    }
                }
                ui.label("📁");
            } else {
                ui.add_space(14.0);
                ui.label("📄");
            }
            if ui.button(name).clicked() {
                if is_dir {
                    if expanded { self.expanded.remove(path); } else { self.expanded.insert(path.to_string()); }
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
        ui.label("Find");
        ui.text_edit_singleline(&mut self.find_text);
        ui.label("Replace");
        ui.text_edit_singleline(&mut self.replace_text);
        if ui.button("Find All").clicked() { self.do_find(); }
        if ui.button("Replace All").clicked() { self.do_replace(); }
        ui.separator();
        let results = self.docs[self.active].find_all(&self.find_text);
        if self.find_text.is_empty() {
            ui.label(egui::RichText::new("Enter text to search").weak());
        } else {
            ui.label(format!("{} matches", results.len()));
            let content = self.docs[self.active].content.clone();
            for (start, _end) in results.iter().take(100) {
                let (line, _) = char_to_line_col(&content, *start);
                let text = content.lines().nth(line - 1).unwrap_or("").trim();
                if ui.button(format!("{}: {}", line, text)).clicked() {
                    self.cursor = (line, 1);
                }
            }
        }
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();
        ui.label("Color theme");
        let themes = self.all_themes();
        let current = self.theme();
        egui::ComboBox::from_label("Theme")
            .selected_text(current.name.clone())
            .show_ui(ui, |ui| {
                for theme in &themes {
                    if ui.selectable_label(theme.id == self.settings.theme_id, &theme.name).clicked() {
                        self.settings.theme_id = theme.id.clone();
                        self.settings_dirty = true;
                    }
                }
            });

        ui.separator();
        ui.checkbox(&mut self.settings.show_line_numbers, "Show line numbers");
        ui.checkbox(&mut self.settings.word_wrap, "Word wrap");
        ui.checkbox(&mut self.settings.insert_spaces, "Insert spaces for Tab");
        if ui.add(egui::Slider::new(&mut self.settings.font_size, 8.0..=32.0).text("Font size")).changed() {
            self.settings_dirty = true;
        }
        if ui.add(egui::Slider::new(&mut self.settings.tab_size, 2..=8).text("Tab size")).changed() {
            self.settings_dirty = true;
        }

        ui.separator();
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

        ui.separator();
        ui.heading("Custom themes");
        if ui.button("Create theme from current").clicked() {
            let mut theme = self.theme();
            let id = format!("custom-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|value| value.as_millis())
                .unwrap_or(0));
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
                        self.settings.theme_id = "light".into();
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
                *field(&mut self.settings.custom_themes[index]) = color_to_hex(egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
                self.settings_dirty = true;
            }
        });
    }

    fn ui_panel(&mut self, ui: &mut egui::Ui) {
        if !self.settings.panel_visible {
            return;
        }
        let build = |ui: &mut egui::Ui, app: &mut Self| {
            ui.horizontal(|ui| {
                if ui.selectable_label(app.panel_view == PanelView::Problems, "Problems").clicked() {
                    app.panel_view = PanelView::Problems;
                }
                if ui.selectable_label(app.panel_view == PanelView::Output, "Output").clicked() {
                    app.panel_view = PanelView::Output;
                }
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
                    for line in &app.output {
                        ui.label(line);
                    }
                }
            }
        };
        match self.settings.panel_position.as_str() {
            "right" => {
                egui::Panel::right("panel_right").default_size(280.0).show(ui, |ui| build(ui, self));
            }
            "left" => {
                egui::Panel::left("panel_left").default_size(280.0).show(ui, |ui| build(ui, self));
            }
            _ => {
                egui::Panel::bottom("panel_bottom").default_size(180.0).show(ui, |ui| build(ui, self));
            }
        }
    }

    fn ui_center(&mut self, ui: &mut egui::Ui) {
        let show_numbers = self.settings.show_line_numbers;
        let font_size = self.settings.font_size;
        let mut new_cursor = None;
        let mut changed = false;

        {
            let doc = &mut self.docs[self.active];
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
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
                    let output = egui::TextEdit::multiline(&mut doc.content)
                        .code_editor()
                        .frame(egui::Frame::NONE)
                        .desired_width(f32::INFINITY)
                        .show(ui);
                    changed = output.response.changed();
                    if let Some(range) = output.cursor_range {
                        new_cursor = Some(char_to_line_col(&doc.content, range.primary.index.0));
                    }
                });
            });
        }

        if changed {
            self.docs[self.active].modified = true;
        }
        if let Some(cursor) = new_cursor {
            self.cursor = cursor;
        }
    }

    fn ui_status(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Ln {}, Col {}", self.cursor.0, self.cursor.1));
                ui.separator();
                let doc = &self.docs[self.active];
                ui.label(if doc.modified { "Modified" } else { "Saved" });
                ui.separator();
                ui.label(format!("Spaces: {}", self.settings.tab_size));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(self.theme().name);
                });
            });
        });
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    let command = modifiers.ctrl || modifiers.command;
                    if !command {
                        continue;
                    }
                    match key {
                        egui::Key::N => self.new_doc(),
                        egui::Key::O => self.open_file(),
                        egui::Key::S => self.save_file(modifiers.shift),
                        egui::Key::F => {
                            self.side_view = SideView::Search;
                            self.settings.side_visible = true;
                        }
                        egui::Key::T => {
                            let themes = self.all_themes();
                            let index = themes.iter().position(|t| t.id == self.settings.theme_id).unwrap_or(0);
                            self.settings.theme_id = themes[(index + 1) % themes.len()].id.clone();
                            self.settings_dirty = true;
                        }
                        egui::Key::B => {
                            self.settings.side_visible = !self.settings.side_visible;
                            self.settings_dirty = true;
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

impl eframe::App for PezliApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx();
        self.apply_style(ctx);
        self.shortcuts(ctx);
        self.ui_menu(ui);
        self.ui_tabs(ui);
        self.ui_side(ui);
        self.ui_panel(ui);
        self.ui_status(ui);
        egui::CentralPanel::no_frame().show(ui, |ui| self.ui_center(ui));

        if self.about_open {
            let mut open = self.about_open;
            egui::Window::new("About Pezli Editor").open(&mut open).resizable(false).show(ctx, |ui| {
                ui.heading("Pezli Editor v1.0.0");
                ui.label("A native desktop text editor built with Rust and egui.");
                ui.label("Features: themes, settings, folders, find and replace, rearrangeable workspace.");
                ui.label("Built by Beauttah Kipruto, MUST.");
            });
            self.about_open = open;
        }

        if self.settings_dirty {
            self.settings.save();
            self.settings_dirty = false;
        }
    }
}
