//! Native AM bridge sound generator: a carrier/modulator oscillator pair
//! connected by the `Am-Synth` bridge.
//!
//! In `Mix` mode the two oscillators are summed; in `AM` mode the carrier is
//! amplitude-modulated by the modulator (carrier * (1 + modulator + depth)),
//! exactly like the per-voice bridge in `crate::voice::Osc2Mode`.

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
pub const MODE: &str = "mode";
pub const AM_DEPTH: &str = "am_depth";

const PARAMS: &[ModuleParamSpec] = &[
    ModuleParamSpec { name: CARRIER_WAVEFORM, default: 0.0 },
    ModuleParamSpec { name: CARRIER_LEVEL, default: 0.7 },
    ModuleParamSpec { name: CARRIER_PITCH, default: 0.0 },
    ModuleParamSpec { name: MODULATOR_WAVEFORM, default: 1.0 },
    ModuleParamSpec { name: MODULATOR_LEVEL, default: 0.5 },
    ModuleParamSpec { name: MODULATOR_PITCH, default: 0.0 },
    ModuleParamSpec { name: MODE, default: 1.0 },
    ModuleParamSpec { name: AM_DEPTH, default: 0.5 },
];

/// `mode`: `0` = Mix (both oscillators summed), `1` = AM (carrier modulated
/// by the modulator), mirroring [`crate::voice::Osc2Mode`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BridgeMode {
    Mix,
    Am,
}

impl BridgeMode {
    fn from_index(index: usize) -> Self {
        match index {
            0 => BridgeMode::Mix,
            _ => BridgeMode::Am,
        }
    }
}

pub struct AmBridgeModule {
    name: String,
    carrier: Oscillator,
    modulator: Oscillator,
    carrier_waveform: Waveform,
    carrier_level: f32,
    carrier_pitch: f32,
    modulator_waveform: Waveform,
    modulator_level: f32,
    modulator_pitch: f32,
    mode: BridgeMode,
    am_depth: f32,
    gate: bool,
}

impl AmBridgeModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            carrier: Oscillator::new(44_100.0, Waveform::Sine),
            modulator: Oscillator::new(44_100.0, Waveform::Saw),
            carrier_waveform: Waveform::Sine,
            carrier_level: 0.7,
            carrier_pitch: 0.0,
            modulator_waveform: Waveform::Saw,
            modulator_level: 0.5,
            modulator_pitch: 0.0,
            mode: BridgeMode::Am,
            am_depth: 0.5,
            gate: false,
        }
    }

    fn tune(&mut self, base_frequency: f32) {
        self.carrier.set_frequency(base_frequency * semitone_mult(self.carrier_pitch));
        self.modulator
            .set_frequency(base_frequency * semitone_mult(self.modulator_pitch));
    }
}

impl AudioModule for AmBridgeModule {
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
            CARRIER_WAVEFORM => self.carrier_waveform = Waveform::from_index(value.clamp(0.0, 7.0) as usize),
            CARRIER_LEVEL => self.carrier_level = value.clamp(0.0, 1.0),
            CARRIER_PITCH => self.carrier_pitch = value,
            MODULATOR_WAVEFORM => {
                self.modulator_waveform = Waveform::from_index(value.clamp(0.0, 7.0) as usize)
            }
            MODULATOR_LEVEL => self.modulator_level = value.clamp(0.0, 1.0),
            MODULATOR_PITCH => self.modulator_pitch = value,
            MODE => self.mode = BridgeMode::from_index(value.clamp(0.0, 1.0) as usize),
            AM_DEPTH => self.am_depth = value.clamp(0.0, 1.0),
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
            MODE => Some(if self.mode == BridgeMode::Am { 1.0 } else { 0.0 }),
            AM_DEPTH => Some(self.am_depth),
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
        let carrier = self.carrier.generate();
        let modulator = self.modulator.generate();
        let sample = match self.mode {
            BridgeMode::Mix => carrier * self.carrier_level + modulator * self.modulator_level,
            BridgeMode::Am => {
                carrier * (1.0 + modulator * self.am_depth) * self.carrier_level
            }
        };
        frame[0] = sample;
        frame[1] = sample;
    }
}

fn semitone_mult(semitones: f32) -> f32 {
    2.0_f32.powf(semitones / 12.0)
}

pub fn register(registry: &mut ModuleRegistry) {
    let builder = Arc::new(|| -> Box<dyn AudioModule> {
        Box::new(AmBridgeModule::new("bridge".into()))
    });
    registry.register("am_bridge", ModuleKind::SoundGen, builder);
}