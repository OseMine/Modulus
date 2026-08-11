//! Native chorus module.

use std::sync::Arc;

use crate::fx::{Chorus, ChorusParams};
use crate::modules::registry::ModuleRegistry;
use crate::modules::{AudioModule, ModuleEvents, ModuleKind, ModuleParamSpec};

pub const DRY_WET: &str = "dry_wet";
pub const DEPTH: &str = "depth";
pub const RATE: &str = "rate";
pub const VOICES: &str = "voices";
pub const DELAY_MS: &str = "delay_ms";
pub const WIDTH: &str = "width";

const PARAMS: &[ModuleParamSpec] = &[
    ModuleParamSpec {
        name: DRY_WET,
        default: 0.35,
    },
    ModuleParamSpec {
        name: DEPTH,
        default: 0.5,
    },
    ModuleParamSpec {
        name: RATE,
        default: 1.0,
    },
    ModuleParamSpec {
        name: VOICES,
        default: 2.0,
    },
    ModuleParamSpec {
        name: DELAY_MS,
        default: 10.0,
    },
    ModuleParamSpec {
        name: WIDTH,
        default: 0.5,
    },
];

pub struct ChorusModule {
    name: String,
    chorus: Chorus,
    dry_wet: f32,
    depth: f32,
    rate: f32,
    voices: f32,
    delay_ms: f32,
    width: f32,
}

impl ChorusModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            chorus: Chorus::new(),
            dry_wet: 0.35,
            depth: 0.5,
            rate: 1.0,
            voices: 2.0,
            delay_ms: 10.0,
            width: 0.5,
        }
    }
}

impl AudioModule for ChorusModule {
    fn kind(&self) -> ModuleKind {
        ModuleKind::Fx
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn rename(&mut self, id: &str) {
        self.name = id.to_string();
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.chorus.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.chorus.reset();
    }

    fn params(&self) -> &[ModuleParamSpec] {
        PARAMS
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            DRY_WET => self.dry_wet = value.clamp(0.0, 1.0),
            DEPTH => self.depth = value.clamp(0.0, 1.0),
            RATE => self.rate = value.clamp(0.1, 10.0),
            VOICES => self.voices = value.clamp(1.0, 8.0),
            DELAY_MS => self.delay_ms = value.max(0.0),
            WIDTH => self.width = value.clamp(0.0, 1.0),
            _ => return false,
        }
        true
    }

    fn param_value(&self, name: &str) -> Option<f32> {
        match name {
            DRY_WET => Some(self.dry_wet),
            DEPTH => Some(self.depth),
            RATE => Some(self.rate),
            VOICES => Some(self.voices),
            DELAY_MS => Some(self.delay_ms),
            WIDTH => Some(self.width),
            _ => None,
        }
    }

    fn process(&mut self, frame: &mut [f32; 2], _events: &ModuleEvents, sample_rate: f32) {
        let params = ChorusParams {
            dry_wet: self.dry_wet,
            depth: self.depth,
            rate: self.rate,
            voices: self.voices as usize,
            delay_ms: self.delay_ms,
            width: self.width,
        };
        self.chorus.process(frame, &params, sample_rate);
    }
}

pub fn register(registry: &mut ModuleRegistry) {
    let builder =
        Arc::new(|| -> Box<dyn AudioModule> { Box::new(ChorusModule::new("chorus".into())) });
    registry.register("chorus", ModuleKind::Fx, builder);
}
