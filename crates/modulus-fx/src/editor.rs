//! Modern egui editor for Modulus FX.

use std::sync::Arc;

use modulus_ui::{dark_visuals, section, slider_row, ACCENT};
use nih_plug::prelude::Editor;
use nih_plug_egui::{create_egui_editor, egui, widgets};

use crate::ModulusFxParams;

pub fn create_editor(params: Arc<ModulusFxParams>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        (),
        |ctx, _| {
            ctx.set_visuals(dark_visuals());
        },
        move |ctx, setter, _| {
            egui::TopBottomPanel::top("header").show(ctx, |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("MODULUS FX")
                            .size(22.0)
                            .strong()
                            .color(ACCENT),
                    );
                    ui.label(
                        egui::RichText::new("multi-effects processor")
                            .size(12.0)
                            .weak(),
                    );
                });
                ui.add_space(8.0);
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add_space(4.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().slider_width = 280.0;

                    section(ui, "Filter", |ui| {
                        slider_row(
                            ui,
                            "Type",
                            widgets::ParamSlider::for_param(&params.filt_type, setter),
                        );
                        slider_row(
                            ui,
                            "Enabled",
                            widgets::ParamSlider::for_param(&params.filt_enabled, setter),
                        );
                        slider_row(
                            ui,
                            "Cutoff",
                            widgets::ParamSlider::for_param(&params.filt_cutoff, setter),
                        );
                        slider_row(
                            ui,
                            "Resonance",
                            widgets::ParamSlider::for_param(&params.filt_resonance, setter),
                        );
                        slider_row(
                            ui,
                            "Smoothing",
                            widgets::ParamSlider::for_param(&params.filt_smoothing, setter),
                        );
                    });

                    section(ui, "Chorus", |ui| {
                        slider_row(
                            ui,
                            "Enabled",
                            widgets::ParamSlider::for_param(&params.chorus_enabled, setter),
                        );
                        slider_row(
                            ui,
                            "Dry/Wet",
                            widgets::ParamSlider::for_param(&params.chorus_dry_wet, setter),
                        );
                        slider_row(
                            ui,
                            "Depth",
                            widgets::ParamSlider::for_param(&params.chorus_depth, setter),
                        );
                        slider_row(
                            ui,
                            "Rate",
                            widgets::ParamSlider::for_param(&params.chorus_rate, setter),
                        );
                        slider_row(
                            ui,
                            "Voices",
                            widgets::ParamSlider::for_param(&params.chorus_voices, setter),
                        );
                        slider_row(
                            ui,
                            "Delay",
                            widgets::ParamSlider::for_param(&params.chorus_delay, setter),
                        );
                        slider_row(
                            ui,
                            "Width",
                            widgets::ParamSlider::for_param(&params.chorus_width, setter),
                        );
                    });

                    section(ui, "Gain", |ui| {
                        slider_row(
                            ui,
                            "Gain In",
                            widgets::ParamSlider::for_param(&params.gain_in, setter),
                        );
                        slider_row(
                            ui,
                            "Gain Out",
                            widgets::ParamSlider::for_param(&params.gain_out, setter),
                        );
                    });
                });
            });
        },
    )
}
