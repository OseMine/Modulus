use std::f32::consts::PI;

use crate::rng::FastRng;
use crate::waveform::Waveform;

/// Phase-accumulator oscillator with a selectable waveform.
///
/// Consolidates the phase-accumulation loop from `variable-synth` and the
/// stateful sine oscillator from `Am-Synth` into a single unit.
pub struct Oscillator {
    waveform: Waveform,
    frequency: f32,
    sample_rate: f32,
    phase: f32,
    phase_increment: f32,
    rng: FastRng,
}

impl Oscillator {
    pub fn new(sample_rate: f32, waveform: Waveform) -> Self {
        let mut osc = Self {
            waveform,
            frequency: 440.0,
            sample_rate,
            phase: 0.0,
            phase_increment: 0.0,
            rng: FastRng::new(0x9E37_79B9 ^ (waveform as u32).wrapping_mul(0x85EB_CA6B)),
        };
        osc.update_phase_increment();
        osc
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.update_phase_increment();
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_phase_increment();
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    fn update_phase_increment(&mut self) {
        self.phase_increment = 2.0 * PI * self.frequency / self.sample_rate;
    }

    pub fn generate(&mut self) -> f32 {
        let sample = self.waveform.generate(self.phase, &mut self.rng);
        self.phase += self.phase_increment;
        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        sample
    }
}
