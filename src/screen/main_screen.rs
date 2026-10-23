
use eframe::egui;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Copy, PartialEq, Default)]
enum Language {
    #[default]
    English,
    German,
}

impl Language {
    fn text(self, english: &'static str, german: &'static str) -> &'static str {
        match self {
            Self::English => english,
            Self::German => german,
        }
    }
}

#[derive(Default)]
pub struct MainApp {
    next_sheet_id: u64,
    sheets: Vec<SheetWindow>,
    language: Language,
}

struct SheetWindow {
    id: u64,
    open: Arc<AtomicBool>,
}

impl eframe::App for MainApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        self.sheets
            .retain(|sheet| sheet.open.load(Ordering::Acquire));

        // Gemeinsames modernes Farbschema.
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = egui::Color32::from_rgb(18, 22, 34);
        style.visuals.window_fill = egui::Color32::from_rgb(25, 30, 45);
        style.visuals.widgets.inactive.bg_fill =
            egui::Color32::from_rgb(39, 48, 68);
        style.visuals.widgets.inactive.fg_stroke.color =
            egui::Color32::from_rgb(225, 232, 245);
        style.visuals.widgets.hovered.bg_fill =
            egui::Color32::from_rgb(60, 83, 120);
        style.visuals.widgets.hovered.fg_stroke.color =
            egui::Color32::WHITE;
        style.visuals.widgets.active.bg_fill =
            egui::Color32::from_rgb(75, 115, 180);
        style.visuals.selection.bg_fill =
            egui::Color32::from_rgb(72, 110, 175);
        style.visuals.window_corner_radius = egui::CornerRadius::same(14);
        style.visuals.widgets.inactive.corner_radius =
            egui::CornerRadius::same(10);
        style.visuals.widgets.hovered.corner_radius =
            egui::CornerRadius::same(10);
        style.spacing.button_padding = egui::vec2(18.0, 12.0);
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        ctx.set_style(style);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(18, 22, 34))
                    .inner_margin(egui::Margin::same(24)),
            )
            .show(ctx, |ui| {
                // Language selector in the top-right area.
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(
                            self.language.text("LANGUAGE", "SPRACHE"),
                        )
                            .small()
                            .color(egui::Color32::from_rgb(160, 174, 198)),
                    );

                    let english = self.language == Language::English;

                    if ui
                        .selectable_label(english, "English")
                        .clicked()
                    {
                        self.language = Language::English;
                    }

                    if ui
                        .selectable_label(!english, "Deutsch")
                        .clicked()
                    {
                        self.language = Language::German;
                    }
                });

                ui.add_space(45.0);

                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("♫")
                            .size(54.0)
                            .color(egui::Color32::from_rgb(112, 160, 245)),
                    );

                    ui.add_space(8.0);

                    ui.heading(
                        egui::RichText::new("Music Sheet Editor")
                            .size(32.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );

                    ui.add_space(8.0);

                    ui.label(
                        egui::RichText::new(self.language.text(
                            "Create, edit and explore your music.",
                            "Erstelle, bearbeite und entdecke deine Musik.",
                        ))
                            .size(15.0)
                            .color(egui::Color32::from_rgb(160, 174, 198)),
                    );

                    ui.add_space(42.0);

                    let button_width = 310.0;
                    let button_height = 58.0;

                    // Primary action.
                    let new_file = egui::Button::new(
                        egui::RichText::new(
                            self.language.text(
                                "＋   New File",
                                "＋   Neue Datei",
                            ),
                        )
                            .size(19.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                        .fill(egui::Color32::from_rgb(65, 112, 195))
                        .corner_radius(egui::CornerRadius::same(13));

                    if ui
                        .add_sized([button_width, button_height], new_file)
                        .clicked()
                    {
                        self.next_sheet_id += 1;

                        self.sheets.push(SheetWindow {
                            id: self.next_sheet_id,
                            open: Arc::new(AtomicBool::new(true)),
                        });
                    }

                    ui.add_space(14.0);

                    let open_file = egui::Button::new(
                        egui::RichText::new(
                            self.language.text(
                                "Open File",
                                "Datei öffnen",
                            ),
                        )
                            .size(17.0),
                    )
                        .fill(egui::Color32::from_rgb(39, 48, 68))
                        .corner_radius(egui::CornerRadius::same(13));

                    if ui
                        .add_sized([button_width, button_height], open_file)
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Music Sheet", &["json"])
                            .pick_file()
                        {
                            // File loading is not implemented yet.
                            eprintln!("Selected: {}", path.display());
                        }
                    }

                    ui.add_space(28.0);

                    ui.label(
                        egui::RichText::new(self.language.text(
                            "Create a new sheet to get started.",
                            "Erstelle eine neue Notenseite, um loszulegen.",
                        ))
                            .small()
                            .color(egui::Color32::from_rgb(125, 140, 166)),
                    );
                });
            });

        // Register every open sheet viewport on every frame.
        for sheet in &self.sheets {
            let id = sheet.id;
            let open = Arc::clone(&sheet.open);
            let language = self.language;
            let title = format!(
                "{} {id}",
                language.text("Music Sheet", "Notenblatt"),
            );

            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of(("music_sheet", id)),
                egui::ViewportBuilder::default()
                    .with_title(title.clone())
                    .with_inner_size([1100.0, 800.0])
                    .with_min_inner_size([700.0, 500.0]),
                move |ctx, _class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        open.store(false, Ordering::Release);
                        return;
                    }

                    // The language argument is added in file_screen.rs.
                    crate::screen::file_screen::sheet_view(
                        ctx,
                        &title,
                        language == Language::English,
                    );
                },
            );
        }
    }
}
