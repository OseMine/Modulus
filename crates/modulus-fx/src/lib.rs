use std::num::NonZeroU32;
use std::sync::Arc;

use modulus_core::fx::{ChorusParams, FxEngine, FxFrameParams};
use nih_plug::prelude::*;

mod editor;
mod params;
use params::ModulusFxParams;

pub struct ModulusFx {
    params: Arc<ModulusFxParams>,
    sample_rate: f32,
    engine: FxEngine,
}

impl Default for ModulusFx {
    fn default() -> Self {
        Self {
            params: Arc::new(ModulusFxParams::default()),
            sample_rate: 44_100.0,
            engine: FxEngine::new(),
        }
    }
}

impl Plugin for ModulusFx {
    const NAME: &'static str = "Modulus FX";
    const VENDOR: &'static str = "OskarFX";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create_editor(self.params.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.engine.set_sample_rate(self.sample_rate);
        true
    }

    fn reset(&mut self) {
        self.engine.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let params = &self.params;
        let sample_rate = self.sample_rate;
        let two_pi = 2.0 * std::f32::consts::PI;

        for mut channel_samples in buffer.iter_samples() {
            let smoothing_ms = params.filt_smoothing.value();
            let smoothing_coeff = if smoothing_ms > 0.0 {
                (-two_pi * (1.0 / (smoothing_ms * 0.001 * sample_rate))).exp()
            } else {
                0.0
            };

            let mut frame = [0.0_f32; 2];
            for (channel_index, sample) in channel_samples.iter_mut().enumerate() {
                if channel_index < 2 {
                    frame[channel_index] = *sample;
                }
            }

            let frame_params = FxFrameParams {
                filter_type: params.filt_type.value().to_core(),
                filter_cutoff: params.filt_cutoff.value(),
                filter_resonance: params.filt_resonance.value(),
                filter_smoothing_coeff: smoothing_coeff,
                filter_enabled: params.filt_enabled.value(),
                gain_in_db: params.gain_in.smoothed.next(),
                chorus_enabled: params.chorus_enabled.value(),
                chorus: ChorusParams {
                    dry_wet: params.chorus_dry_wet.smoothed.next(),
                    depth: params.chorus_depth.smoothed.next(),
                    rate: params.chorus_rate.smoothed.next(),
                    voices: params.chorus_voices.value() as usize,
                    delay_ms: params.chorus_delay.value(),
                    width: params.chorus_width.smoothed.next(),
                },
                gain_out_db: params.gain_out.smoothed.next(),
            };

            self.engine.process(&mut frame, &frame_params, sample_rate);

            for (channel_index, sample) in channel_samples.iter_mut().enumerate() {
                if channel_index < 2 {
                    *sample = frame[channel_index];
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for ModulusFx {
    const CLAP_ID: &'static str = "com.the-muzikar.modulus-fx";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Multi-effects processor");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Filter,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for ModulusFx {
    const VST3_CLASS_ID: [u8; 16] = *b"ModulusFXPlugin.";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Filter,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(ModulusFx);
nih_export_vst3!(ModulusFx);
