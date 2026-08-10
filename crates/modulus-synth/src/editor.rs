//! Modern egui editor for the Modulus synthesizer.
//!
//! The layout uses expanded/contracted sections for each voice section:
//! Oscillators, Filter, Amp Envelope, Filter Envelope, Chorus and Output.
//! All control rows use the nih-plug `ParamSlider` widget, which knows how
//! to display and edit `FloatParam`/`IntParam`/`EnumParam`/`BoolParam`
//! values directly.

use std::sync::Arc;

use nih_plug::prelude::Editor;
use nih_plug_egui::{create_egui_editor, egui, widgets};

use crate::ModulusParams;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x5c, 0xb6, 0xff);
const BG: egui::Color32 = egui::Color32::from_rgb(0x14, 0x16, 0x1a);
const PANEL: egui::Color32 = egui::Color32::from_rgb(0x1c, 0x1f, 0x26);

/// One labeled slider row inside a section.
fn slider_row(ui: &mut egui::Ui, label: &str, widget: impl egui::Widget) {
    ui.label(egui::RichText::new(label).weak());
    ui.add(widget);
    ui.end_row();
}

/// A section header followed by a two-column grid of labeled sliders.
fn section(ui: &mut egui::Ui, title: &str, rows: impl FnOnce(&mut egui::Ui)) {
    ui.label(egui::RichText::new(title).strong().color(ACCENT));
    ui.indent(title, |ui| {
        egui::Grid::new(title)
            .num_columns(2)
            .spacing([24.0, 10.0])
            .show(ui, rows);
    });
    ui.add_space(10.0);
}

pub fn create_editor(
    params: Arc<ModulusParams>,
    design_state: Arc<crate::DesignState>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        (),
        |ctx, _| {
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = PANEL;
            visuals.window_fill = BG;
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(0x2a, 0x2e, 0x37);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x36, 0x3c, 0x47);
            visuals.widgets.active.bg_fill = ACCENT;
            visuals.selection.bg_fill = ACCENT;
            ctx.set_visuals(visuals);
        },
        move |ctx, setter, _| {
            egui::TopBottomPanel::top("header").show(ctx, |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("MODULUS")
                            .size(22.0)
                            .strong()
                            .color(ACCENT),
                    );
                    ui.label(
                        egui::RichText::new("hybrid subtractive synthesizer")
                            .size(12.0)
                            .weak(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let voices =
                            design_state.voice_count.load(std::sync::atomic::Ordering::Relaxed);
                        let color = if voices > 0 {
                            ACCENT
                        } else {
                            egui::Color32::from_rgb(0x3a, 0x40, 0x4a)
                        };
                        ui.colored_label(color, format!("{voices} voices"));
                    });
                });
                ui.add_space(8.0);
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add_space(4.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().slider_width = 280.0;

                    section(ui, "Global", |ui| {
                        slider_row(ui, "Tuning", widgets::ParamSlider::for_param(&params.global_tuning, setter));
                    });

                    section(ui, "Oscillator 1", |ui| {
                        slider_row(ui, "Waveform", widgets::ParamSlider::for_param(&params.osc1_waveform, setter));
                        slider_row(ui, "Level", widgets::ParamSlider::for_param(&params.osc1_level, setter));
                        slider_row(ui, "Pitch", widgets::ParamSlider::for_param(&params.osc1_pitch, setter));
                    });

                    section(ui, "Oscillator 2", |ui| {
                        slider_row(ui, "Waveform", widgets::ParamSlider::for_param(&params.osc2_waveform, setter));
                        slider_row(ui, "Level", widgets::ParamSlider::for_param(&params.osc2_level, setter));
                        slider_row(ui, "Pitch", widgets::ParamSlider::for_param(&params.osc2_pitch, setter));
                        slider_row(ui, "Mode", widgets::ParamSlider::for_param(&params.osc2_mode, setter));
                        slider_row(ui, "AM Depth", widgets::ParamSlider::for_param(&params.osc2_am_depth, setter));
                    });

                    section(ui, "Filter", |ui| {
                        slider_row(ui, "Type", widgets::ParamSlider::for_param(&params.filt_type, setter));
                        slider_row(ui, "Cutoff", widgets::ParamSlider::for_param(&params.filt_cutoff, setter));
                        slider_row(ui, "Resonance", widgets::ParamSlider::for_param(&params.filt_resonance, setter));
                        slider_row(ui, "Env Amount", widgets::ParamSlider::for_param(&params.filt_env_amount, setter));
                    });

                    section(ui, "Amp Envelope", |ui| {
                        slider_row(ui, "Attack", widgets::ParamSlider::for_param(&params.env_attack, setter));
                        slider_row(ui, "Decay", widgets::ParamSlider::for_param(&params.env_decay, setter));
                        slider_row(ui, "Sustain", widgets::ParamSlider::for_param(&params.env_sustain, setter));
                        slider_row(ui, "Release", widgets::ParamSlider::for_param(&params.env_release, setter));
                    });

                    section(ui, "Filter Envelope", |ui| {
                        slider_row(ui, "Attack", widgets::ParamSlider::for_param(&params.fenv_attack, setter));
                        slider_row(ui, "Decay", widgets::ParamSlider::for_param(&params.fenv_decay, setter));
                        slider_row(ui, "Sustain", widgets::ParamSlider::for_param(&params.fenv_sustain, setter));
                        slider_row(ui, "Release", widgets::ParamSlider::for_param(&params.fenv_release, setter));
                    });

                    section(ui, "Chorus / Output", |ui| {
                        slider_row(ui, "Enable", widgets::ParamSlider::for_param(&params.fx_enable, setter));
                        slider_row(ui, "Dry/Wet", widgets::ParamSlider::for_param(&params.fx_chorus_dry_wet, setter));
                        slider_row(ui, "Depth", widgets::ParamSlider::for_param(&params.fx_chorus_depth, setter));
                        slider_row(ui, "Rate", widgets::ParamSlider::for_param(&params.fx_chorus_rate, setter));
                        slider_row(ui, "Voices", widgets::ParamSlider::for_param(&params.fx_chorus_voices, setter));
                        slider_row(ui, "Delay", widgets::ParamSlider::for_param(&params.fx_chorus_delay, setter));
                        slider_row(ui, "Width", widgets::ParamSlider::for_param(&params.fx_chorus_width, setter));
                        slider_row(ui, "Output Gain", widgets::ParamSlider::for_param(&params.fx_gain, setter));
                    });
                });
            });
        },
    )
}