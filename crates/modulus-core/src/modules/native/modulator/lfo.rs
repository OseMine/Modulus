//! Native low-frequency oscillator (LFO) module: a free-running modulator
//! that scales the frame by `1 - depth + depth * waveform` (unipolar, 0..1).
//!
//! At `depth = 0` the module is a passthrough; at `depth = 1` the frame is
//! fully swelled between silence and unity at the LFO rate (tremolo).

use std::sync::Arc;

use crate::modules::registry::ModuleRegistry;
use crate::modules::{AudioModule, ModuleEvents, ModuleKind, ModuleParamSpec};
use crate::oscillator::Oscillator;
use crate::waveform::Waveform;

pub const WAVEFORM: &str = "waveform";
pub const RATE_HZ: &str = "rate_hz";
pub const DEPTH: &str = "depth";

const PARAMS: &[ModuleParamSpec] = &[
    ModuleParamSpec { name: WAVEFORM, default: 0.0 },
    ModuleParamSpec { name: RATE_HZ, default: 1.0 },
    ModuleParamSpec { name: DEPTH, default: 0.5 },
];

pub struct LfoModule {
    name: String,
    osc: Oscillator,
    waveform: Waveform,
    rate_hz: f32,
    depth: f32,
}

impl LfoModule {
    pub fn new(name: String) -> Self {
        let mut osc = Oscillator::new(44_100.0, Waveform::Sine);
        osc.set_frequency(1.0);
        Self {
            name,
            osc,
            waveform: Waveform::Sine,
            rate_hz: 1.0,
            depth: 0.5,
        }
    }
}

impl AudioModule for LfoModule {
    fn kind(&self) -> ModuleKind {
        ModuleKind::Modulator
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn rename(&mut self, id: &str) {
        self.name = id.to_string();
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.osc.set_sample_rate(sample_rate);
        self.osc.set_frequency(self.rate_hz);
    }

    fn reset(&mut self) {
        self.osc.reset();
    }

    fn params(&self) -> &[ModuleParamSpec] {
        PARAMS
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            WAVEFORM => self.waveform = Waveform::from_index(value.clamp(0.0, 7.0) as usize),
            RATE_HZ => {
                self.rate_hz = value.clamp(0.01, 20.0);
                self.osc.set_frequency(self.rate_hz);
            }
            DEPTH => self.depth = value.clamp(0.0, 1.0),
            _ => return false,
        }
        true
    }

    fn param_value(&self, name: &str) -> Option<f32> {
        match name {
            WAVEFORM => Some(self.waveform as u8 as f32),
            RATE_HZ => Some(self.rate_hz),
            DEPTH => Some(self.depth),
            _ => None,
        }
    }

    fn process(&mut self, frame: &mut [f32; 2], _events: &ModuleEvents, _sample_rate: f32) {
        self.osc.set_waveform(self.waveform);
        let bipolar = self.osc.generate();
        let unipolar = ((bipolar + 1.0) * 0.5).clamp(0.0, 1.0);
        let gain = 1.0 - self.depth + self.depth * unipolar;
        frame[0] *= gain;
        frame[1] *= gain;
    }
}

pub fn register(registry: &mut ModuleRegistry) {
    let builder = Arc::new(|| -> Box<dyn AudioModule> { Box::new(LfoModule::new("lfo".into())) });
    registry.register("lfo", ModuleKind::Modulator, builder);
}