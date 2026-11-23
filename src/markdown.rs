use crate::tools::toggle_checklist;
use eframe::egui::{self, Color32};

pub fn render_markdown(ui: &mut egui::Ui, content: &mut String, font_size: f32, accent: Color32) {
    let lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    let mut toggles: Vec<usize> = Vec::new();
    let mut in_code = false;
    let mut code = String::new();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if in_code {
                code_panel(ui, &code, font_size);
                code.clear();
            }
            in_code = !in_code;
            continue;
        }
        if in_code {
            if !code.is_empty() {
                code.push('\n');
            }
            code.push_str(line);
            continue;
        }

        if trimmed.is_empty() {
            ui.add_space(font_size * 0.4);
            continue;
        }
        if trimmed.starts_with("---") || trimmed.starts_with("***") {
            ui.separator();
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("### ") {
            ui.add(egui::Label::new(
                egui::RichText::new(rest).size(font_size * 1.15).strong().color(accent),
            ));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            ui.add(egui::Label::new(
                egui::RichText::new(rest).size(font_size * 1.4).strong().color(accent),
            ));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            ui.add(egui::Label::new(
                egui::RichText::new(rest).size(font_size * 1.8).strong().color(accent),
            ));
            continue;
        }

        let task = if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            Some((false, rest))
        } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            Some((true, rest))
        } else if let Some(rest) = trimmed.strip_prefix("- [X] ") {
            Some((true, rest))
        } else {
            None
        };
        if let Some((checked, rest)) = task {
            ui.horizontal(|ui| {
                let mut state = checked;
                if ui.checkbox(&mut state, "").clicked() {
                    toggles.push(index);
                }
                add_inline(ui, rest, font_size);
            });
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("> ") {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.add(egui::Label::new(
                    egui::RichText::new(rest).italics().weak(),
                ));
            });
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("+ "))
        {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("•").color(accent));
                add_inline(ui, rest, font_size);
            });
            continue;
        }

        let mut numbered = trimmed.splitn(2, ". ");
        let first = numbered.next().unwrap_or("");
        if !first.is_empty() && first.chars().all(|c| c.is_ascii_digit()) {
            if let Some(rest) = numbered.next() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("{}.", first)).color(accent));
                    add_inline(ui, rest, font_size);
                });
                continue;
            }
        }

        add_inline(ui, trimmed, font_size);
    }
    if in_code {
        code_panel(ui, &code, font_size);
    }

    toggles.sort_unstable_by(|a, b| b.cmp(a));
    for index in toggles {
        toggle_checklist(content, index + 1);
    }
}

fn code_panel(ui: &mut egui::Ui, code: &str, font_size: f32) {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .inner_margin(egui::Margin::same(8))
        .corner_radius(egui::CornerRadius::same(4))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(code).monospace().size(font_size * 0.9));
        });
}

fn add_inline(ui: &mut egui::Ui, text: &str, font_size: f32) {
    ui.horizontal_wrapped(|ui| {
        let chars: Vec<char> = text.chars().collect();
        let mut buffer = String::new();
        let mut index = 0usize;
        while index < chars.len() {
            let rest: String = chars[index..].iter().collect();
            if rest.starts_with("**") {
                flush(ui, &buffer, font_size, Style::Normal);
                buffer.clear();
                if let Some(end) = find_marker(&chars, index + 2, "**") {
                    let inner: String = chars[index + 2..end].iter().collect();
                    flush(ui, &inner, font_size, Style::Bold);
                    index = end + 2;
                    continue;
                }
            }
            if rest.starts_with('`') && !rest.starts_with("```") {
                flush(ui, &buffer, font_size, Style::Normal);
                buffer.clear();
                if let Some(end) = find_char(&chars, index + 1, '`') {
                    let inner: String = chars[index + 1..end].iter().collect();
                    flush(ui, &inner, font_size, Style::Code);
                    index = end + 1;
                    continue;
                }
            }
            if rest.starts_with('*') {
                flush(ui, &buffer, font_size, Style::Normal);
                buffer.clear();
                if let Some(end) = find_char(&chars, index + 1, '*') {
                    let inner: String = chars[index + 1..end].iter().collect();
                    flush(ui, &inner, font_size, Style::Italic);
                    index = end + 1;
                    continue;
                }
            }
            if rest.starts_with('[') {
                if let Some(mid) = find_char(&chars, index + 1, ']') {
                    if mid + 1 < chars.len() && chars[mid + 1] == '(' {
                        if let Some(end) = find_char(&chars, mid + 2, ')') {
                            flush(ui, &buffer, font_size, Style::Normal);
                            buffer.clear();
                            let label: String = chars[index + 1..mid].iter().collect();
                            let url: String = chars[mid + 2..end].iter().collect();
                            let _ = url;
                            flush(ui, &label, font_size, Style::Link);
                            index = end + 1;
                            continue;
                        }
                    }
                }
            }
            buffer.push(chars[index]);
            index += 1;
        }
        flush(ui, &buffer, font_size, Style::Normal);
    });
}

#[derive(Clone, Copy)]
enum Style {
    Normal,
    Bold,
    Italic,
    Code,
    Link,
}

fn flush(ui: &mut egui::Ui, text: &str, font_size: f32, style: Style) {
    if text.is_empty() {
        return;
    }
    let mut rich = egui::RichText::new(text).size(font_size);
    rich = match style {
        Style::Normal => rich,
        Style::Bold => rich.strong(),
        Style::Italic => rich.italics(),
        Style::Code => rich.monospace().background_color(ui.visuals().faint_bg_color),
        Style::Link => rich.color(ui.visuals().hyperlink_color).underline(),
    };
    ui.label(rich);
}

fn find_marker(chars: &[char], from: usize, marker: &str) -> Option<usize> {
    let needle: Vec<char> = marker.chars().collect();
    if chars.len() < from + needle.len() {
        return None;
    }
    (from..=chars.len() - needle.len()).find(|&i| chars[i..i + needle.len()] == needle[..])
}

fn find_char(chars: &[char], from: usize, target: char) -> Option<usize> {
    (from..chars.len()).find(|&i| chars[i] == target)
}
