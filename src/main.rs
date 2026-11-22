mod app;
mod editor;
mod settings;
mod theme;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([700.0, 480.0])
            .with_title("Pezli Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "Pezli Editor",
        options,
        Box::new(|cc| Ok(Box::new(app::PezliApp::new(cc)) as Box<dyn eframe::App>)),
    )
}
