use std::num::NonZeroU32;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use modulus_core::fx::{ChorusParams, SynthFxParams};
use modulus_core::synth::SynthEngine;
use modulus_core::voice::SynthFrameParams;
use nih_plug::prelude::*;

mod editor;
mod params;
use params::ModulusParams;

/// Live state shared with the GUI: the number of currently sounding voices.
pub struct DesignState {
    pub voice_count: AtomicUsize,
}

impl DesignState {
    pub fn new() -> Self {
        Self {
            voice_count: AtomicUsize::new(0),
        }
    }
}

impl Default for DesignState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Modulus {
    params: Arc<ModulusParams>,
    sample_rate: f32,
    engine: SynthEngine,
    design_state: Arc<DesignState>,
}

impl Default for Modulus {
    fn default() -> Self {
        Self {
            params: Arc::new(ModulusParams::default()),
            sample_rate: 44_100.0,
            engine: SynthEngine::new(44_100.0),
            design_state: Arc::new(DesignState::new()),
        }
    }
}

impl Plugin for Modulus {
    const NAME: &'static str = "Modulus";
    const VENDOR: &'static str = "OskarFX";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create_editor(self.params.clone(), self.design_state.clone())
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
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let params = &self.params;
        let sample_rate = self.sample_rate;
        let mut next_event = context.next_event();

        for (sample_id, mut channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        self.engine.note_on(note, velocity);
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        self.engine.note_off(note);
                    }
                    _ => (),
                }
                next_event = context.next_event();
            }

            let frame_params = SynthFrameParams {
                osc1_waveform: params.osc1_waveform.value().to_core(),
                osc1_level: params.osc1_level.smoothed.next(),
                osc1_pitch_semitones: params.osc1_pitch.value(),
                osc2_waveform: params.osc2_waveform.value().to_core(),
                osc2_level: params.osc2_level.smoothed.next(),
                osc2_pitch_semitones: params.osc2_pitch.value(),
                osc2_mode: params.osc2_mode.value().to_core(),
                osc2_am_depth: params.osc2_am_depth.smoothed.next(),
                filter_type: params.filt_type.value().to_core(),
                filter_cutoff: params.filt_cutoff.smoothed.next(),
                filter_resonance: params.filt_resonance.smoothed.next(),
                filter_env_amount: params.filt_env_amount.smoothed.next(),
                amp_attack: params.env_attack.value(),
                amp_decay: params.env_decay.value(),
                amp_sustain: params.env_sustain.value(),
                amp_release: params.env_release.value(),
                filt_attack: params.fenv_attack.value(),
                filt_decay: params.fenv_decay.value(),
                filt_sustain: params.fenv_sustain.value(),
                filt_release: params.fenv_release.value(),
                tuning_hz: params.global_tuning.value(),
            };

            let fx_params = SynthFxParams {
                chorus_enabled: params.fx_enable.value(),
                chorus: ChorusParams {
                    dry_wet: params.fx_chorus_dry_wet.smoothed.next(),
                    depth: params.fx_chorus_depth.smoothed.next(),
                    rate: params.fx_chorus_rate.smoothed.next(),
                    voices: params.fx_chorus_voices.value() as usize,
                    delay_ms: params.fx_chorus_delay.value(),
                    width: params.fx_chorus_width.smoothed.next(),
                },
                gain_db: params.fx_gain.smoothed.next(),
            };

            let frame = self.engine.process(&frame_params, &fx_params, sample_rate);
            if self.params.editor_state.is_open() {
                self.design_state
                    .voice_count
                    .store(self.engine.active_voices(), Ordering::Relaxed);
            }
            for (channel_index, sample) in channel_samples.iter_mut().enumerate() {
                *sample = frame[channel_index.min(1)];
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Modulus {
    const CLAP_ID: &'static str = "com.the-muzikar.modulus";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Hybrid subtractive synthesizer");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for Modulus {
    const VST3_CLASS_ID: [u8; 16] = *b"ModulusSynth....";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_export_clap!(Modulus);
nih_export_vst3!(Modulus);
