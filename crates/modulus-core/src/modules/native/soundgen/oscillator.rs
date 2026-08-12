//! Native oscillator module: any of the eight Modulus waveforms.

use std::sync::Arc;

use crate::midi::midi_note_to_freq;
use crate::modules::registry::ModuleRegistry;
use crate::modules::{AudioModule, ModuleEvents, ModuleKind, ModuleParamSpec};
use crate::oscillator::Oscillator;
use crate::waveform::Waveform;

pub const WAVEFORM: &str = "waveform";
pub const LEVEL: &str = "level";
pub const PITCH: &str = "pitch_semitones";

const PARAMS: &[ModuleParamSpec] = &[
    ModuleParamSpec {
        name: WAVEFORM,
        default: 0.0,
    },
    ModuleParamSpec {
        name: LEVEL,
        default: 0.7,
    },
    ModuleParamSpec {
        name: PITCH,
        default: 0.0,
    },
];

pub struct OscillatorModule {
    name: String,
    osc: Oscillator,
    waveform: Waveform,
    level: f32,
    pitch_semitones: f32,
}

impl OscillatorModule {
    pub fn new(name: String) -> Self {
        let waveform = Waveform::Sine;
        Self {
            name,
            osc: Oscillator::new(44_100.0, waveform),
            waveform,
            level: 0.7,
            pitch_semitones: 0.0,
        }
    }
}

impl AudioModule for OscillatorModule {
    fn kind(&self) -> ModuleKind {
        ModuleKind::SoundGen
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn rename(&mut self, id: &str) {
        self.name = id.to_string();
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.osc.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.osc.reset();
    }

    fn note_on(&mut self, note: u8, _velocity: f32, tuning_hz: f32) {
        let semitones = 2.0_f32.powf(self.pitch_semitones / 12.0);
        self.osc
            .set_frequency(midi_note_to_freq(note, tuning_hz) * semitones);
    }

    fn note_off(&mut self, _note: u8) {}

    fn params(&self) -> &[ModuleParamSpec] {
        PARAMS
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            WAVEFORM => {
                let waveform = Waveform::from_index(value.clamp(0.0, 7.0) as usize);
                self.waveform = waveform;
                self.osc.set_waveform(waveform);
            }
            LEVEL => self.level = value.clamp(0.0, 1.0),
            PITCH => self.pitch_semitones = value.clamp(-24.0, 24.0),
            _ => return false,
        }
        true
    }

    fn param_value(&self, name: &str) -> Option<f32> {
        match name {
            WAVEFORM => Some(self.waveform as u8 as f32),
            LEVEL => Some(self.level),
            PITCH => Some(self.pitch_semitones),
            _ => None,
        }
    }

    fn process(&mut self, frame: &mut [f32; 2], _events: &ModuleEvents, _sample_rate: f32) {
        // Sources keep generating after note_off: the amp envelope in the
        // signal chain shapes the release tail.
        let sample = self.osc.generate() * self.level;
        frame[0] = sample;
        frame[1] = sample;
    }
}

pub fn register(registry: &mut ModuleRegistry) {
    let builder =
        Arc::new(|| -> Box<dyn AudioModule> { Box::new(OscillatorModule::new("osc".into())) });
    registry.register("oscillator", ModuleKind::SoundGen, builder);
    let builder =
        Arc::new(|| -> Box<dyn AudioModule> { Box::new(OscillatorModule::new("osc2".into())) });
    registry.register("oscillator2", ModuleKind::SoundGen, builder);
}
