use eframe::egui;
use egui::{Color32, FontFamily, FontId, Pos2, Stroke};
use serde::{Deserialize, Serialize};

use printpdf::{BuiltinFont, Line, Mm, PdfDocument, Point};

const STAFF_COUNT: usize = 10;
const STAFF_SPACING: f32 = 100.0;
const LINE_SPACING: f32 = 10.0;
const NOTE_STEP: f32 = 5.0;
const NOTE_X_STEP: f32 = 18.0;

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum NoteType {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

impl NoteType {
    fn label(self) -> &'static str {
        match self {
            Self::Whole => "Ganze",
            Self::Half => "Halbe",
            Self::Quarter => "Viertel",
            Self::Eighth => "Achtel",
            Self::Sixteenth => "Sechzehntel",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct PlacedNote {
    x_step: i32,
    y_step: i32,
    staff_index: usize,
    note_type: NoteType,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct PlacedBarline {
    x_step: i32,
    staff_index: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct SheetSettings {
    numerator: String,
    denominator: String,
    clef: String,
    clef_popup: bool,
    meter_popup: bool,
    selected_note: NoteType,
    place_barline_mode: bool,
    notes: Vec<PlacedNote>,
    barlines: Vec<PlacedBarline>,
}

impl Default for SheetSettings {
    fn default() -> Self {
        Self {
            numerator: "4".into(),
            denominator: "4".into(),
            clef: "Violin".into(),
            clef_popup: false,
            meter_popup: false,
            selected_note: NoteType::Quarter,
            place_barline_mode: false,
            notes: Vec::new(),
            barlines: Vec::new(),
        }
    }
}

fn parse_numerator(text: &str) -> Option<u8> {
    text.parse::<u8>()
        .ok()
        .filter(|n| (1..=32).contains(n))
}

fn parse_denominator(text: &str) -> Option<u8> {
    text.parse::<u8>()
        .ok()
        .filter(|n| [1, 2, 4, 8, 16, 32].contains(n))
}

fn music_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("Bravura".into()))
}

fn draw_clef(painter: &egui::Painter, x: f32, y: f32, clef: &str) {
    let symbol = match clef {
        "Bass" => '\u{E062}',
        "Alto" => '\u{E05C}',
        _ => '\u{E050}',
    };
    painter.text(
        egui::pos2(x, y),
        egui::Align2::CENTER_CENTER,
        symbol.to_string(),
        music_font(46.0),
        Color32::BLACK,
    );
}

fn draw_time_signature(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    numerator: u8,
    denominator: u8,
) {
    let font = FontId::new(30.0, FontFamily::Proportional);
    painter.text(
        egui::pos2(x, y - 13.0),
        egui::Align2::CENTER_CENTER,
        numerator.to_string(),
        font.clone(),
        Color32::BLACK,
    );
    painter.text(
        egui::pos2(x, y + 13.0),
        egui::Align2::CENTER_CENTER,
        denominator.to_string(),
        font,
        Color32::BLACK,
    );
}

// Draw ledger lines *above* the staff (y_step > 8). Staff top line is at y_step = 8.
// We draw at y_step = 10, 12, 14, ... stepping by 2 (every other step).

fn draw_ledger_lines_top(
    painter: &egui::Painter,
    x: f32,
    y_step: i32,
    staff_top: f32,
) {
    if y_step <= 8 {
        return;
    }

    let half_width = 8.0;
    let stroke = Stroke::new(1.0, Color32::BLACK);

    // Oberhalb des Systems: 10, 12, 14, ...
    for step in (10..=y_step).step_by(2) {
        let line_y = staff_top - (step - 8) as f32 * NOTE_STEP;

        painter.line_segment(
            [
                Pos2::new(x - half_width, line_y),
                Pos2::new(x + half_width, line_y),
            ],
            stroke,
        );
    }
}

fn draw_ledger_lines_bottom(
    painter: &egui::Painter,
    x: f32,
    y_step: i32,
    staff_bottom: f32,
) {
    if y_step >= 0 {
        return;
    }

    let half_width = 8.0;
    let stroke = Stroke::new(1.0, Color32::BLACK);

    // Unterhalb des Systems: -2, -4, -6, ...
    // Zeichne die Linien von der unteren Notenlinie nach unten.
    let ledger_count = (-y_step) / 2;

    for index in 1..=ledger_count {
        let line_y = staff_bottom + index as f32 * 2.0 * NOTE_STEP;

        painter.line_segment(
            [
                Pos2::new(x - half_width, line_y),
                Pos2::new(x + half_width, line_y),
            ],
            stroke,
        );
    }
}



fn draw_note(painter: &egui::Painter, note: PlacedNote, x: f32, y: f32) {
    let head = egui::Rect::from_center_size(
        egui::pos2(x, y),
        egui::vec2(12.0, 8.0),
    );
    let stem_x = head.right() - 1.0;
    let filled = matches!(
        note.note_type,
        NoteType::Quarter | NoteType::Eighth | NoteType::Sixteenth
    );
    if filled {
        painter.add(egui::Shape::ellipse_filled(
            egui::pos2(x, y),
            egui::vec2(6.0, 4.0),
            Color32::BLACK,
        ));
    } else {
        painter.add(egui::Shape::ellipse_stroke(
            egui::pos2(x, y),
            egui::vec2(6.0, 4.0),
            Stroke::new(1.5, Color32::BLACK),
        ));
    }
    if note.note_type == NoteType::Whole {
        return;
    }
    let stem_top = y - 34.0;
    painter.line_segment(
        [egui::pos2(stem_x, y), egui::pos2(stem_x, stem_top)],
        Stroke::new(1.5, Color32::BLACK),
    );
    let flag_count = match note.note_type {
        NoteType::Eighth => 1,
        NoteType::Sixteenth => 2,
        _ => 0,
    };
    for flag in 0..flag_count {
        let flag_y = stem_top + flag as f32 * 7.0;
        painter.add(egui::Shape::line(
            vec![
                egui::pos2(stem_x, flag_y),
                egui::pos2(stem_x + 9.0, flag_y + 7.0),
                egui::pos2(stem_x + 7.0, flag_y + 13.0),
            ],
            Stroke::new(1.5, Color32::BLACK),
        ));
    }
}


fn tr<'a>(english: bool, german: &'a str, english_text: &'a str) -> &'a str {
    if english {
        english_text
    } else {
        german
    }
}

fn toolbar_button(
    ui: &mut egui::Ui,
    label: &str,
    active: bool,
) -> egui::Response {
    let (fill, text) = if active {
        (
            Color32::from_rgb(65, 105, 180),
            Color32::WHITE,
        )
    } else {
        (
            Color32::from_rgb(48, 51, 58),
            Color32::from_rgb(235, 238, 243),
        )
    };

    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .size(13.0)
                .color(text),
        )
            .fill(fill)
            .corner_radius(9.0)
            .min_size(egui::vec2(0.0, 36.0))
            .stroke(egui::Stroke::new(
                1.0,
                Color32::from_rgb(70, 74, 83),
            )),
    )
}

fn note_label(kind: NoteType, english: bool) -> &'static str {
    match (kind, english) {
        (NoteType::Whole, false) => "Ganze",
        (NoteType::Half, false) => "Halbe",
        (NoteType::Quarter, false) => "Viertel",
        (NoteType::Eighth, false) => "Achtel",
        (NoteType::Sixteenth, false) => "Sechzehntel",
        (NoteType::Whole, true) => "Whole",
        (NoteType::Half, true) => "Half",
        (NoteType::Quarter, true) => "Quarter",
        (NoteType::Eighth, true) => "Eighth",
        (NoteType::Sixteenth, true) => "Sixteenth",
    }
}



fn save_sheet(settings: &SheetSettings) -> Result<(), String> {
    use printpdf::{
        BuiltinFont, Line, Mm, PdfDocument, Point,
    };

    let Some(path) = rfd::FileDialog::new()
        .add_filter("PDF-Datei", &["pdf"])
        .set_file_name("music_sheet.pdf")
        .save_file()
    else {
        return Ok(());
    };

    let (doc, page, layer) = PdfDocument::new(
        "Music Sheet",
        Mm(210.0),
        Mm(297.0),
        "Partitur",
    );

    let current_layer = doc.get_page(page).get_layer(layer);

    let regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| e.to_string())?;

    let bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| e.to_string())?;

    // Die Zeichnung basiert auf einem 1000 x 1200 Pixel großen
    // egui-Notenblatt und wird proportional auf A4 skaliert.
    let scale = 0.21_f32;
    let offset_y = 274.5_f32;

    let pdf_x = |x: f32| Mm(x * scale);
    let pdf_y = |y: f32| Mm(offset_y - y * scale);

    let draw_line = |x1: f32, y1: f32, x2: f32, y2: f32| {
        current_layer.add_line(Line {
            points: vec![
                (Point::new(pdf_x(x1), pdf_y(y1)), false),
                (Point::new(pdf_x(x2), pdf_y(y2)), false),
            ],
            is_closed: false,
        });
    };

    let left = 45.0_f32;
    let right = 965.0_f32;
    let note_start_x = left + 125.0;

    for staff_index in 0..STAFF_COUNT {
        let top = 120.0 + staff_index as f32 * STAFF_SPACING;
        let bottom = top + 4.0 * LINE_SPACING;
        let center_y = top + 20.0;

        // Fünf Notenlinien
        for line in 0..5 {
            let y = top + line as f32 * LINE_SPACING;
            draw_line(left, y, right, y);
        }

        // Notenschlüssel: G, F oder C als druckbares Ersatzzeichen.
        // Für echte Musikglyphen ist eine eingebettete Bravura-Schrift nötig.
        let clef_label = match settings.clef.as_str() {
            "Bass" => "F",
            "Alto" => "C",
            _ => "G",
        };

        current_layer.use_text(
            clef_label,
            28.0,
            pdf_x(left + 15.0),
            pdf_y(center_y + 12.0),
            &bold,
        );

        // Taktart
        if let (Some(n), Some(d)) = (
            parse_numerator(&settings.numerator),
            parse_denominator(&settings.denominator),
        ) {
            current_layer.use_text(
                n.to_string(),
                16.0,
                pdf_x(left + 67.0),
                pdf_y(center_y - 1.0),
                &bold,
            );

            current_layer.use_text(
                d.to_string(),
                16.0,
                pdf_x(left + 67.0),
                pdf_y(center_y + 15.0),
                &bold,
            );
        }

        // Taktstriche
        for barline in settings
            .barlines
            .iter()
            .filter(|b| b.staff_index == staff_index)
        {
            let x = note_start_x + barline.x_step as f32 * NOTE_X_STEP;
            draw_line(x, top - 1.0, x, bottom + 1.0);
        }

        // Noten und Hilfslinien
        for note in settings
            .notes
            .iter()
            .filter(|n| n.staff_index == staff_index)
        {
            let x = note_start_x + note.x_step as f32 * NOTE_X_STEP;
            let y = bottom - note.y_step as f32 * NOTE_STEP;

            // Hilfslinien oberhalb des Systems
            if note.y_step > 8 {
                for step in (10..=note.y_step).step_by(2) {
                    let line_y = top - (step - 8) as f32 * NOTE_STEP;
                    draw_line(x - 8.0, line_y, x + 8.0, line_y);
                }
            }

            // Hilfslinien unterhalb des Systems
            if note.y_step < 0 {
                for index in 1..=(-note.y_step / 2) {
                    let line_y = bottom + index as f32 * 2.0 * NOTE_STEP;
                    draw_line(x - 8.0, line_y, x + 8.0, line_y);
                }
            }

            // Notenkopf als kleines Vieleck
            let filled = matches!(
                note.note_type,
                NoteType::Quarter | NoteType::Eighth | NoteType::Sixteenth
            );

            let points: Vec<(Point, bool)> = (0..24)
                .map(|i| {
                    let angle = i as f32
                        * std::f32::consts::TAU / 24.0;

                    let px = x + angle.cos() * 6.0;
                    let py = y + angle.sin() * 4.0;

                    (Point::new(pdf_x(px), pdf_y(py)), false)
                })
                .collect();

            current_layer.add_line(Line {
                points,
                is_closed: true,
            });

            if filled {
                // Schwarzer Mittelpunkt für gefüllte Notenköpfe
                current_layer.use_text(
                    "●",
                    8.0,
                    pdf_x(x - 2.0),
                    pdf_y(y - 2.0),
                    &regular,
                );
            }

            // Ganze Noten haben keinen Notenhals.
            if note.note_type == NoteType::Whole {
                continue;
            }

            let stem_x = x + 5.0;
            let stem_top = y - 34.0;
            draw_line(stem_x, y, stem_x, stem_top);

            let flag_count = match note.note_type {
                NoteType::Eighth => 1,
                NoteType::Sixteenth => 2,
                _ => 0,
            };

            for flag in 0..flag_count {
                let flag_y = stem_top + flag as f32 * 7.0;
                draw_line(
                    stem_x,
                    flag_y,
                    stem_x + 9.0,
                    flag_y + 7.0,
                );
                draw_line(
                    stem_x + 9.0,
                    flag_y + 7.0,
                    stem_x + 7.0,
                    flag_y + 13.0,
                );
            }
        }
    }

    let file = std::fs::File::create(path)
        .map_err(|e| e.to_string())?;

    let mut writer = std::io::BufWriter::new(file);

    doc.save(&mut writer)
        .map_err(|e| e.to_string())?;

    Ok(())
}


pub fn sheet_view(
    ctx: &egui::Context,
    title: &str,
    english: bool,
) {
    let id = egui::Id::new(("music_sheet_settings", title));

    let mut settings = ctx.data_mut(|data| {
        data.get_temp::<SheetSettings>(id).unwrap_or_default()
    });

    egui::CentralPanel::default()
        .frame(
            egui::Frame::default()
                .fill(Color32::from_rgb(224, 226, 230)),
        )
        .show(ctx, |ui| {
            // Einheitliche Werkzeugleiste
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading(title);
                    ui.separator();

                    if toolbar_button(
                        ui,
                        tr(english, "Notenschlüssel", "Clef"),
                        settings.clef_popup,
                    )
                        .clicked()
                    {
                        settings.clef_popup = !settings.clef_popup;
                    }

                    if toolbar_button(
                        ui,
                        tr(english, "Taktart", "Time Signature"),
                        settings.meter_popup,
                    )
                        .clicked()
                    {
                        settings.meter_popup = !settings.meter_popup;
                    }

                    if toolbar_button(
                        ui,
                        tr(english, "Als PDF speichern", "Save as PDF"),
                        false,
                    )
                        .clicked()
                    {
                        if let Err(error) = save_sheet(&settings) {
                            eprintln!("PDF export error: {error}");
                        }
                    }

                    ui.separator();

                    ui.label(tr(
                        english,
                        "Notenwert:",
                        "Note value:",
                    ));

                    for kind in [
                        NoteType::Whole,
                        NoteType::Half,
                        NoteType::Quarter,
                        NoteType::Eighth,
                        NoteType::Sixteenth,
                    ] {
                        if toolbar_button(
                            ui,
                            note_label(kind, english),
                            settings.selected_note == kind
                                && !settings.place_barline_mode,
                        )
                            .clicked()
                        {
                            settings.selected_note = kind;
                            settings.place_barline_mode = false;
                        }
                    }

                    if toolbar_button(
                        ui,
                        tr(
                            english,
                            "Letzte Note löschen",
                            "Undo Note",
                        ),
                        false,
                    )
                        .clicked()
                    {
                        settings.notes.pop();
                    }

                    if toolbar_button(
                        ui,
                        tr(
                            english,
                            "Alle Noten löschen",
                            "Clear Notes",
                        ),
                        false,
                    )
                        .clicked()
                    {
                        settings.notes.clear();
                    }
                });

                ui.add_space(6.0);

                ui.horizontal_wrapped(|ui| {
                    let mode_label = if settings.place_barline_mode {
                        tr(
                            english,
                            "Taktstrich-Modus: AN",
                            "Barline Mode: ON",
                        )
                    } else {
                        tr(
                            english,
                            "Taktstrich platzieren",
                            "Place Barline",
                        )
                    };

                    if toolbar_button(
                        ui,
                        mode_label,
                        settings.place_barline_mode,
                    )
                        .clicked()
                    {
                        settings.place_barline_mode =
                            !settings.place_barline_mode;
                    }

                    if toolbar_button(
                        ui,
                        tr(
                            english,
                            "Letzten Taktstrich löschen",
                            "Undo Barline",
                        ),
                        false,
                    )
                        .clicked()
                    {
                        settings.barlines.pop();
                    }

                    if toolbar_button(
                        ui,
                        tr(
                            english,
                            "Alle Taktstriche löschen",
                            "Clear Barlines",
                        ),
                        false,
                    )
                        .clicked()
                    {
                        settings.barlines.clear();
                    }
                });

                ui.add_space(4.0);

                let hint = if settings.place_barline_mode {
                    tr(
                        english,
                        "Klicke auf ein Notensystem, um einen Taktstrich zu setzen.",
                        "Click a staff to place a barline.",
                    )
                } else {
                    tr(
                        english,
                        "Wähle einen Notenwert und klicke auf das Notenblatt.",
                        "Select a note value and click on the sheet.",
                    )
                };

                ui.label(
                    egui::RichText::new(hint)
                        .size(12.0)
                        .color(Color32::from_gray(100)),
                );
            });

            ui.separator();

            // Notenschlüssel-Dialog
            if settings.clef_popup {
                egui::Window::new(tr(
                    english,
                    "Notenschlüssel auswählen",
                    "Select Clef",
                ))
                    .id(id.with("clef_popup"))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        for (clef, label_de, label_en) in [
                            ("Violin", "Violinschlüssel", "Treble Clef"),
                            ("Bass", "Bassschlüssel", "Bass Clef"),
                            ("Alto", "Altschlüssel", "Alto Clef"),
                        ] {
                            if ui
                                .button(tr(english, label_de, label_en))
                                .clicked()
                            {
                                settings.clef = clef.to_owned();
                                settings.clef_popup = false;
                            }
                        }
                    });
            }

            // Taktart-Dialog
            if settings.meter_popup {
                egui::Window::new(tr(
                    english,
                    "Taktart einstellen",
                    "Set Time Signature",
                ))
                    .id(id.with("meter_popup"))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(tr(
                            english,
                            "Zähler (1–32)",
                            "Numerator (1–32)",
                        ));

                        ui.add(
                            egui::TextEdit::singleline(
                                &mut settings.numerator,
                            )
                                .desired_width(120.0)
                                .char_limit(2),
                        );

                        ui.add_space(8.0);

                        ui.label(tr(
                            english,
                            "Nenner (1, 2, 4, 8, 16, 32)",
                            "Denominator (1, 2, 4, 8, 16, 32)",
                        ));

                        ui.add(
                            egui::TextEdit::singleline(
                                &mut settings.denominator,
                            )
                                .desired_width(120.0)
                                .char_limit(2),
                        );

                        let valid =
                            parse_numerator(&settings.numerator).is_some()
                                && parse_denominator(
                                &settings.denominator,
                            )
                                .is_some();

                        if !valid {
                            ui.colored_label(
                                Color32::from_rgb(200, 55, 55),
                                tr(
                                    english,
                                    "Bitte gültige Werte eingeben.",
                                    "Please enter valid values.",
                                ),
                            );
                        }

                        let enter =
                            ui.input(|i| i.key_pressed(egui::Key::Enter));

                        let apply = ui
                            .add_enabled(
                                valid,
                                egui::Button::new(tr(
                                    english,
                                    "Übernehmen",
                                    "Apply",
                                ))
                                    .corner_radius(8.0),
                            )
                            .clicked();

                        if valid && (enter || apply) {
                            settings.meter_popup = false;
                        }
                    });
            }

            // Notenblatt
            egui::ScrollArea::both().show(ui, |ui| {
                let available = ui.available_size();
                let page_width = available.x.max(1000.0);
                let page_height = available.y.max(1200.0);

                let (page_rect, response) = ui.allocate_exact_size(
                    egui::vec2(page_width, page_height),
                    egui::Sense::click(),
                );

                let painter = ui.painter_at(page_rect);

                painter.rect_filled(
                    page_rect,
                    2.0,
                    Color32::WHITE,
                );

                painter.rect_stroke(
                    page_rect,
                    2.0,
                    Stroke::new(
                        1.0,
                        Color32::from_rgb(205, 208, 214),
                    ),
                    egui::StrokeKind::Inside,
                );

                let left = page_rect.left() + 45.0;
                let right = page_rect.right() - 35.0;
                let first_staff_y = page_rect.top() + 120.0;
                let note_start_x = left + 125.0;

                for staff_index in 0..STAFF_COUNT {
                    let top = first_staff_y
                        + staff_index as f32 * STAFF_SPACING;
                    let bottom = top + 4.0 * LINE_SPACING;
                    let center_y = top + 20.0;

                    // Fünf Notenlinien
                    for line in 0..5 {
                        let line_y =
                            top + line as f32 * LINE_SPACING;

                        painter.line_segment(
                            [
                                egui::pos2(left, line_y),
                                egui::pos2(right, line_y),
                            ],
                            Stroke::new(1.0, Color32::BLACK),
                        );
                    }

                    draw_clef(
                        &painter,
                        left + 22.0,
                        center_y,
                        &settings.clef,
                    );

                    if let (Some(n), Some(d)) = (
                        parse_numerator(&settings.numerator),
                        parse_denominator(&settings.denominator),
                    ) {
                        draw_time_signature(
                            &painter,
                            left + 75.0,
                            center_y,
                            n,
                            d,
                        );
                    }

                    // Taktstriche zeichnen
                    for barline in settings
                        .barlines
                        .iter()
                        .filter(|b| b.staff_index == staff_index)
                    {
                        let bar_x = note_start_x
                            + barline.x_step as f32 * NOTE_X_STEP;

                        painter.line_segment(
                            [
                                egui::pos2(bar_x, top - 1.0),
                                egui::pos2(bar_x, bottom + 1.0),
                            ],
                            Stroke::new(1.5, Color32::BLACK),
                        );
                    }

                    // Noten und Hilfslinien zeichnen
                    for note in settings
                        .notes
                        .iter()
                        .filter(|n| n.staff_index == staff_index)
                    {
                        let x = note_start_x
                            + note.x_step as f32 * NOTE_X_STEP;
                        let y = bottom
                            - note.y_step as f32 * NOTE_STEP;

                        if note.y_step < 0 {
                            draw_ledger_lines_bottom(
                                &painter,
                                x,
                                note.y_step,
                                bottom,
                            );
                        } else if note.y_step > 8 {
                            draw_ledger_lines_top(
                                &painter,
                                x,
                                note.y_step,
                                top,
                            );
                        }

                        draw_note(&painter, *note, x, y);
                    }
                }

                // Klicks auf dem Notenblatt verarbeiten
                if response.clicked() {
                    if let Some(pointer) =
                        response.interact_pointer_pos()
                    {
                        if pointer.x >= note_start_x
                            && pointer.x < right
                        {
                            let staff_index = (0..STAFF_COUNT)
                                .min_by_key(|&i| {
                                    let top = first_staff_y
                                        + i as f32 * STAFF_SPACING;
                                    let bottom =
                                        top + 4.0 * LINE_SPACING;

                                    if pointer.y < top {
                                        (top - pointer.y) as i32
                                    } else if pointer.y > bottom {
                                        (pointer.y - bottom) as i32
                                    } else {
                                        0
                                    }
                                })
                                .unwrap_or(0);

                            let top = first_staff_y
                                + staff_index as f32 * STAFF_SPACING;
                            let bottom = top + 4.0 * LINE_SPACING;

                            let x_step = (
                                (pointer.x - note_start_x)
                                    / NOTE_X_STEP
                            )
                                .round() as i32;

                            if settings.place_barline_mode {
                                if pointer.y >= top - 8.0
                                    && pointer.y <= bottom + 8.0
                                {
                                    let exists = settings
                                        .barlines
                                        .iter()
                                        .any(|b| {
                                            b.staff_index == staff_index
                                                && b.x_step == x_step
                                        });

                                    if !exists && x_step >= 0 {
                                        settings.barlines.push(
                                            PlacedBarline {
                                                x_step,
                                                staff_index,
                                            },
                                        );
                                    }
                                }
                            } else {
                                let y_step = (
                                    (bottom - pointer.y) / NOTE_STEP
                                )
                                    .round() as i32;

                                let snapped_y = bottom
                                    - y_step as f32 * NOTE_STEP;

                                if (pointer.y - snapped_y).abs()
                                    <= NOTE_STEP * 1.5
                                    && pointer.y >= top - 60.0
                                    && pointer.y <= bottom + 60.0
                                {
                                    let occupied = settings
                                        .notes
                                        .iter()
                                        .any(|n| {
                                            n.staff_index == staff_index
                                                && n.x_step == x_step
                                                && n.y_step == y_step
                                        });

                                    if !occupied {
                                        settings.notes.push(PlacedNote {
                                            x_step,
                                            y_step,
                                            staff_index,
                                            note_type: settings.selected_note,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            });
        });

    ctx.data_mut(|data| {
        data.insert_temp(id, settings);
    });
}
