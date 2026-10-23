
mod screen;

use eframe::egui;
use std::sync::Arc;

fn setup_music_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "Bravura".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/Bravura.otf"
        ))),
    );

    fonts.families.insert(
        egui::FontFamily::Name("Bravura".into()),
        vec!["Bravura".to_owned()],
    );

    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Notenblatt-Editor",
        options,
        Box::new(|cc| {
            setup_music_font(&cc.egui_ctx);
            Ok(Box::new(screen::main_screen::MainApp::default()))
        }),
    )
}
