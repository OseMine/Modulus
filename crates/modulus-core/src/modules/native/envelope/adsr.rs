//! Native envelope module: a linear ADSR that multiplies into the frame.

use std::sync::Arc;

use crate::envelope::Adsr;
use crate::modules::registry::ModuleRegistry;
use crate::modules::{AudioModule, ModuleEvents, ModuleKind, ModuleParamSpec};

pub const ATTACK: &str = "attack";
pub const DECAY: &str = "decay";
pub const SUSTAIN: &str = "sustain";
pub const RELEASE: &str = "release";

const PARAMS: &[ModuleParamSpec] = &[
    ModuleParamSpec {
        name: ATTACK,
        default: 0.01,
    },
    ModuleParamSpec {
        name: DECAY,
        default: 0.1,
    },
    ModuleParamSpec {
        name: SUSTAIN,
        default: 0.5,
    },
    ModuleParamSpec {
        name: RELEASE,
        default: 0.1,
    },
];

pub struct EnvelopeModule {
    name: String,
    env: Adsr,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    sample_rate: f32,
    last_amp: f32,
}

impl EnvelopeModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            env: Adsr::new(44_100.0),
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.1,
            sample_rate: 44_100.0,
            last_amp: 0.0,
        }
    }

    fn update_env(&mut self) {
        self.env
            .set_params(self.attack, self.decay, self.sustain, self.release);
    }
}

impl AudioModule for EnvelopeModule {
    fn kind(&self) -> ModuleKind {
        ModuleKind::Envelope
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn rename(&mut self, id: &str) {
        self.name = id.to_string();
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.env.set_sample_rate(sample_rate);
        self.update_env();
    }

    fn reset(&mut self) {
        self.env = Adsr::new(self.sample_rate);
        self.update_env();
    }

    fn note_on(&mut self, _note: u8, _velocity: f32, _tuning_hz: f32) {
        self.env.trigger();
    }

    fn note_off(&mut self, _note: u8) {
        self.env.release();
    }

    fn params(&self) -> &[ModuleParamSpec] {
        PARAMS
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            ATTACK => self.attack = value.max(0.001),
            DECAY => self.decay = value.max(0.001),
            SUSTAIN => self.sustain = value.clamp(0.0, 1.0),
            RELEASE => self.release = value.max(0.001),
            _ => return false,
        }
        true
    }

    fn param_value(&self, name: &str) -> Option<f32> {
        match name {
            ATTACK => Some(self.attack),
            DECAY => Some(self.decay),
            SUSTAIN => Some(self.sustain),
            RELEASE => Some(self.release),
            _ => None,
        }
    }

    fn process(&mut self, frame: &mut [f32; 2], _events: &ModuleEvents, _sample_rate: f32) {
        let amp = self.env.process();
        self.last_amp = amp;
        frame[0] *= amp;
        frame[1] *= amp;
    }

    fn cv(&self) -> f32 {
        self.last_amp
    }
}

pub fn register(registry: &mut ModuleRegistry) {
    let builder =
        Arc::new(|| -> Box<dyn AudioModule> { Box::new(EnvelopeModule::new("env".into())) });
    registry.register("envelope", ModuleKind::Envelope, builder);
}
