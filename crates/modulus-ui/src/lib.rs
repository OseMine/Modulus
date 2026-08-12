//! Shared egui helpers for the Modulus plugin editors (synth + FX).
//!
//! Both editors use the same dark theme colors and the same layout helpers
//! (`slider_row`/`section`), so they live here instead of being duplicated.

use nih_plug_egui::egui;

pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x5c, 0xb6, 0xff);
pub const BG: egui::Color32 = egui::Color32::from_rgb(0x14, 0x16, 0x1a);
pub const PANEL: egui::Color32 = egui::Color32::from_rgb(0x1c, 0x1f, 0x26);

/// The dark visuals setup shared by both editors.
pub fn dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PANEL;
    visuals.window_fill = BG;
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(0x2a, 0x2e, 0x37);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x36, 0x3c, 0x47);
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.selection.bg_fill = ACCENT;
    visuals
}

/// One labeled slider row inside a section.
pub fn slider_row(ui: &mut egui::Ui, label: &str, widget: impl egui::Widget) {
    ui.label(egui::RichText::new(label).weak());
    ui.add(widget);
    ui.end_row();
}

/// A section header followed by a two-column grid of labeled sliders.
pub fn section(ui: &mut egui::Ui, title: &str, rows: impl FnOnce(&mut egui::Ui)) {
    ui.label(egui::RichText::new(title).strong().color(ACCENT));
    ui.indent(title, |ui| {
        egui::Grid::new(title)
            .num_columns(2)
            .spacing([24.0, 10.0])
            .show(ui, rows);
    });
    ui.add_space(10.0);
}
