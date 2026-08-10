//! Native gain module: simple dB gain staging.

use std::sync::Arc;

use crate::fx::gain_db_to_linear;
use crate::modules::registry::ModuleRegistry;
use crate::modules::{AudioModule, ModuleEvents, ModuleKind, ModuleParamSpec};

pub const GAIN_DB: &str = "gain_db";

const PARAMS: &[ModuleParamSpec] = &[ModuleParamSpec {
    name: GAIN_DB,
    default: 0.0,
}];

pub struct GainModule {
    name: String,
    gain_db: f32,
}

impl GainModule {
    pub fn new(name: String) -> Self {
        Self { name, gain_db: 0.0 }
    }
}

impl AudioModule for GainModule {
    fn kind(&self) -> ModuleKind {
        ModuleKind::Fx
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn rename(&mut self, id: &str) {
        self.name = id.to_string();
    }

    fn prepare(&mut self, _sample_rate: f32) {}

    fn reset(&mut self) {}

    fn params(&self) -> &[ModuleParamSpec] {
        PARAMS
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            GAIN_DB => self.gain_db = value.clamp(-60.0, 12.0),
            _ => return false,
        }
        true
    }

    fn param_value(&self, name: &str) -> Option<f32> {
        match name {
            GAIN_DB => Some(self.gain_db),
            _ => None,
        }
    }

    fn process(&mut self, frame: &mut [f32; 2], _events: &ModuleEvents, _sample_rate: f32) {
        let gain = gain_db_to_linear(self.gain_db);
        frame[0] *= gain;
        frame[1] *= gain;
    }
}

pub fn register(registry: &mut ModuleRegistry) {
    let builder = Arc::new(|| -> Box<dyn AudioModule> { Box::new(GainModule::new("gain".into())) });
    registry.register("gain", ModuleKind::Fx, builder);
}