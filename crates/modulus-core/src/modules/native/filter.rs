//! Native filter module: one of the four ladder models.

use std::sync::Arc;

use crate::filter::{FilterType, VariableFilter};
use crate::modules::registry::ModuleRegistry;
use crate::modules::{AudioModule, ModuleEvents, ModuleKind, ModuleParamSpec};

pub const TYPE: &str = "filter_type";
pub const CUTOFF: &str = "cutoff";
pub const RESONANCE: &str = "resonance";
pub const SMOOTHING: &str = "smoothing_ms";

const PARAMS: &[ModuleParamSpec] = &[
    ModuleParamSpec { name: TYPE, default: 0.0 },
    ModuleParamSpec { name: CUTOFF, default: 1000.0 },
    ModuleParamSpec { name: RESONANCE, default: 0.3 },
    ModuleParamSpec { name: SMOOTHING, default: 0.0 },
];

pub struct FilterModule {
    name: String,
    filter: VariableFilter,
    filter_type: FilterType,
    cutoff: f32,
    resonance: f32,
    smoothing_ms: f32,
    sample_rate: f32,
}

impl FilterModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            filter: VariableFilter::new(),
            filter_type: FilterType::Moog,
            cutoff: 1000.0,
            resonance: 0.3,
            smoothing_ms: 0.0,
            sample_rate: 44_100.0,
        }
    }

    fn update_smoothing(&mut self) {
        let coeff = if self.smoothing_ms > 0.0 {
            (-std::f32::consts::TAU * (1.0 / (self.smoothing_ms * 0.001 * self.sample_rate))).exp()
        } else {
            0.0
        };
        self.filter.set_smoothing(coeff);
    }
}

impl AudioModule for FilterModule {
    fn kind(&self) -> ModuleKind {
        ModuleKind::Filter
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn rename(&mut self, id: &str) {
        self.name = id.to_string();
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.filter.set_params(self.cutoff, self.resonance);
        self.update_smoothing();
    }

    fn reset(&mut self) {
        self.filter.reset();
    }

    fn params(&self) -> &[ModuleParamSpec] {
        PARAMS
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            TYPE => self.filter_type = FilterType::from_index(value.clamp(0.0, 3.0) as usize),
            CUTOFF => self.cutoff = value.clamp(20.0, 20_000.0),
            RESONANCE => self.resonance = value.clamp(0.0, 1.0),
            SMOOTHING => self.smoothing_ms = value.max(0.0),
            _ => return false,
        }
        true
    }

    fn param_value(&self, name: &str) -> Option<f32> {
        match name {
            TYPE => Some(self.filter_type as u8 as f32),
            CUTOFF => Some(self.cutoff),
            RESONANCE => Some(self.resonance),
            SMOOTHING => Some(self.smoothing_ms),
            _ => None,
        }
    }

    fn process(&mut self, frame: &mut [f32; 2], _events: &ModuleEvents, sample_rate: f32) {
        self.filter.set_params(self.cutoff, self.resonance);
        frame[0] = self.filter.process(frame[0], sample_rate);
        frame[1] = self.filter.process(frame[1], sample_rate);
    }
}

pub fn register(registry: &mut ModuleRegistry) {
    let builder = Arc::new(|| -> Box<dyn AudioModule> { Box::new(FilterModule::new("filter".into())) });
    registry.register("filter", ModuleKind::Filter, builder);
}