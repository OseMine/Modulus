//! Native classic FM bridge sound generator: a 2-operator (DX7-style) pair
//! where the modulator phase-modulates the carrier,
//! `sample = carrier(carrier_phase + modulator * modulator_level * fm_amount)`.
//!
//! Operator frequencies are derived from the note via `pitch` semitone
//! offsets, so integer DX7-like ratios are `+12` (2:1), `+19` (3:1),
//! `+24` (4:1), `+34` (~7:1), and `-12` (1:2).

use std::sync::Arc;

use crate::midi::midi_note_to_freq;
use crate::modules::registry::ModuleRegistry;
use crate::modules::{AudioModule, ModuleEvents, ModuleKind, ModuleParamSpec};
use crate::oscillator::Oscillator;
use crate::waveform::Waveform;

pub const CARRIER_WAVEFORM: &str = "carrier_waveform";
pub const CARRIER_LEVEL: &str = "carrier_level";
pub const CARRIER_PITCH: &str = "carrier_pitch";
pub const MODULATOR_WAVEFORM: &str = "modulator_waveform";
pub const MODULATOR_LEVEL: &str = "modulator_level";
pub const MODULATOR_PITCH: &str = "modulator_pitch";
pub const FM_AMOUNT: &str = "fm_amount";

const PARAMS: &[ModuleParamSpec] = &[
    ModuleParamSpec {
        name: CARRIER_WAVEFORM,
        default: 0.0,
    },
    ModuleParamSpec {
        name: CARRIER_LEVEL,
        default: 0.7,
    },
    ModuleParamSpec {
        name: CARRIER_PITCH,
        default: 0.0,
    },
    ModuleParamSpec {
        name: MODULATOR_WAVEFORM,
        default: 0.0,
    },
    ModuleParamSpec {
        name: MODULATOR_LEVEL,
        default: 0.5,
    },
    ModuleParamSpec {
        name: MODULATOR_PITCH,
        default: 12.0,
    },
    ModuleParamSpec {
        name: FM_AMOUNT,
        default: 0.5,
    },
];

pub struct FmBridgeModule {
    name: String,
    carrier: Oscillator,
    modulator: Oscillator,
    carrier_waveform: Waveform,
    carrier_level: f32,
    carrier_pitch: f32,
    modulator_waveform: Waveform,
    modulator_level: f32,
    modulator_pitch: f32,
    fm_amount: f32,
    gate: bool,
}

impl FmBridgeModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            carrier: Oscillator::new(44_100.0, Waveform::Sine),
            modulator: Oscillator::new(44_100.0, Waveform::Sine),
            carrier_waveform: Waveform::Sine,
            carrier_level: 0.7,
            carrier_pitch: 0.0,
            modulator_waveform: Waveform::Sine,
            modulator_level: 0.5,
            modulator_pitch: 12.0,
            fm_amount: 0.5,
            gate: false,
        }
    }

    fn tune(&mut self, base_frequency: f32) {
        self.carrier
            .set_frequency(base_frequency * semitone_mult(self.carrier_pitch));
        self.modulator
            .set_frequency(base_frequency * semitone_mult(self.modulator_pitch));
    }
}

impl AudioModule for FmBridgeModule {
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
        self.carrier.set_sample_rate(sample_rate);
        self.modulator.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.carrier.reset();
        self.modulator.reset();
        self.gate = false;
    }

    fn note_on(&mut self, note: u8, _velocity: f32, tuning_hz: f32) {
        self.tune(midi_note_to_freq(note, tuning_hz));
        self.gate = true;
    }

    fn note_off(&mut self, _note: u8) {
        self.gate = false;
    }

    fn params(&self) -> &[ModuleParamSpec] {
        PARAMS
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            CARRIER_WAVEFORM => {
                self.carrier_waveform = Waveform::from_index(value.clamp(0.0, 7.0) as usize)
            }
            CARRIER_LEVEL => self.carrier_level = value.clamp(0.0, 1.0),
            CARRIER_PITCH => self.carrier_pitch = value,
            MODULATOR_WAVEFORM => {
                self.modulator_waveform = Waveform::from_index(value.clamp(0.0, 7.0) as usize)
            }
            MODULATOR_LEVEL => self.modulator_level = value.clamp(0.0, 1.0),
            MODULATOR_PITCH => self.modulator_pitch = value,
            FM_AMOUNT => self.fm_amount = value.max(0.0),
            _ => return false,
        }
        true
    }

    fn param_value(&self, name: &str) -> Option<f32> {
        match name {
            CARRIER_WAVEFORM => Some(self.carrier_waveform as u8 as f32),
            CARRIER_LEVEL => Some(self.carrier_level),
            CARRIER_PITCH => Some(self.carrier_pitch),
            MODULATOR_WAVEFORM => Some(self.modulator_waveform as u8 as f32),
            MODULATOR_LEVEL => Some(self.modulator_level),
            MODULATOR_PITCH => Some(self.modulator_pitch),
            FM_AMOUNT => Some(self.fm_amount),
            _ => None,
        }
    }

    fn process(&mut self, frame: &mut [f32; 2], _events: &ModuleEvents, _sample_rate: f32) {
        if !self.gate {
            frame[0] = 0.0;
            frame[1] = 0.0;
            return;
        }
        self.carrier.set_waveform(self.carrier_waveform);
        self.modulator.set_waveform(self.modulator_waveform);
        let modulator = self.modulator.generate();
        let carrier = self
            .carrier
            .generate_at(modulator * self.modulator_level * self.fm_amount);
        let sample = carrier * self.carrier_level;
        frame[0] = sample;
        frame[1] = sample;
    }
}

fn semitone_mult(semitones: f32) -> f32 {
    2.0_f32.powf(semitones / 12.0)
}

pub fn register(registry: &mut ModuleRegistry) {
    let builder =
        Arc::new(|| -> Box<dyn AudioModule> { Box::new(FmBridgeModule::new("fm".into())) });
    registry.register("fm_bridge", ModuleKind::SoundGen, builder);
}
