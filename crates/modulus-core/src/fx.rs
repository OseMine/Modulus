use std::f32::consts::PI;

use crate::filter::FilterType;

const TWO_PI: f32 = 2.0 * PI;

/// Ring-buffer headroom for the chorus at any supported sample rate.
pub const MAX_DELAY_MS: f32 = 100.0;

pub fn gain_db_to_linear(gain_db: f32) -> f32 {
    10.0_f32.powf(gain_db / 20.0)
}

pub fn apply_gain_db(frame: &mut [f32; 2], gain_db: f32) {
    let gain = gain_db_to_linear(gain_db);
    frame[0] *= gain;
    frame[1] *= gain;
}

#[derive(Clone, Copy, Debug)]
pub struct ChorusParams {
    pub dry_wet: f32,
    pub depth: f32,
    pub rate: f32,
    pub voices: usize,
    pub delay_ms: f32,
    pub width: f32,
}

/// Multi-voice modulated-delay chorus.
///
/// The ring buffers are sized in `set_sample_rate` (called from the
/// plugin's `initialize()` hook); `process` never allocates.
pub struct Chorus {
    buffer_left: Vec<f32>,
    buffer_right: Vec<f32>,
    write_left: usize,
    write_right: usize,
    lfo_left: f32,
    lfo_right: f32,
    sample_rate: f32,
    size: usize,
    mask: usize,
}

impl Chorus {
    pub fn new() -> Self {
        Self {
            buffer_left: Vec::new(),
            buffer_right: Vec::new(),
            write_left: 0,
            write_right: 0,
            lfo_left: 0.0,
            lfo_right: PI,
            sample_rate: 0.0,
            size: 0,
            mask: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        if sample_rate == self.sample_rate && !self.buffer_left.is_empty() {
            return;
        }
        self.sample_rate = sample_rate;
        let length = ((MAX_DELAY_MS * 0.001 * sample_rate).ceil() as usize)
            .max(2)
            .next_power_of_two();
        self.size = length;
        self.mask = length - 1;
        self.buffer_left.resize(length, 0.0);
        self.buffer_right.resize(length, 0.0);
        self.reset();
    }

    pub fn reset(&mut self) {
        for sample in &mut self.buffer_left {
            *sample = 0.0;
        }
        for sample in &mut self.buffer_right {
            *sample = 0.0;
        }
        self.write_left = 0;
        self.write_right = 0;
        self.lfo_left = 0.0;
        self.lfo_right = PI;
    }

    #[allow(clippy::needless_range_loop)]
    pub fn process(&mut self, frame: &mut [f32; 2], params: &ChorusParams, sample_rate: f32) {
        if self.size == 0 || params.voices == 0 || params.dry_wet <= 0.0 {
            return;
        }

        let voices = params.voices.clamp(1, 8);
        let max_delay_samples = (MAX_DELAY_MS * 0.001 * sample_rate - 2.0).max(1.0);
        let base_delay = (params.delay_ms * 0.001 * sample_rate).clamp(0.0, max_delay_samples);
        let depth_samples = params.depth.clamp(0.0, 1.0) * max_delay_samples * 0.5;
        let lfo_step = TWO_PI * params.rate / sample_rate;
        let channel_offset = PI * params.width.clamp(0.0, 1.0);
        let voice_spread = TWO_PI / voices as f32;
        let dry = 1.0 - params.dry_wet;
        let wet_gain = params.dry_wet / voices as f32;
        let size = self.size;
        let mask = self.mask;

        for channel in 0..2 {
            let phase = if channel == 0 {
                &mut self.lfo_left
            } else {
                &mut self.lfo_right
            };
            *phase += lfo_step;
            if *phase >= TWO_PI {
                *phase -= TWO_PI;
            }
            let base_phase = *phase + if channel == 1 { channel_offset } else { 0.0 };

            let buffer = if channel == 0 {
                &self.buffer_left
            } else {
                &self.buffer_right
            };
            let write = if channel == 0 {
                &mut self.write_left
            } else {
                &mut self.write_right
            };

            let mut wet = 0.0;
            for voice in 0..voices {
                let voice_phase = base_phase + voice as f32 * voice_spread;
                let modulation = 0.5 + 0.5 * voice_phase.sin();
                let delay = (base_delay + depth_samples * modulation).clamp(1.0, max_delay_samples);
                let delay_floor = delay.floor();
                let fraction = delay - delay_floor;
                let index = (*write + size - delay_floor as usize) & mask;
                let next_index = (index + 1) & mask;
                let delayed =
                    buffer[index] * (1.0 - fraction) + buffer[next_index] * fraction;
                wet += delayed;
            }
            wet *= wet_gain;

            let input = frame[channel];
            let buffer = if channel == 0 {
                &mut self.buffer_left
            } else {
                &mut self.buffer_right
            };
            buffer[*write] = input;
            *write = (*write + 1) & mask;

            frame[channel] = input * dry + wet;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FxFrameParams {
    pub filter_type: FilterType,
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub filter_smoothing_coeff: f32,
    pub filter_enabled: bool,
    pub gain_in_db: f32,
    pub chorus: ChorusParams,
    pub chorus_enabled: bool,
    pub gain_out_db: f32,
}

/// Serial effects rack for Modulus FX: input gain -> variable filter ->
/// chorus -> output gain.
pub struct FxEngine {
    filter: crate::filter::VariableFilter,
    chorus: Chorus,
}

impl FxEngine {
    pub fn new() -> Self {
        Self {
            filter: crate::filter::VariableFilter::new(),
            chorus: Chorus::new(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.chorus.set_sample_rate(sample_rate);
    }

    pub fn reset(&mut self) {
        self.filter.reset();
        self.chorus.reset();
    }

    pub fn process(&mut self, frame: &mut [f32; 2], params: &FxFrameParams, sample_rate: f32) {
        apply_gain_db(frame, params.gain_in_db);

        if params.filter_enabled {
            self.filter.set_type(params.filter_type);
            self.filter.set_smoothing(params.filter_smoothing_coeff);
            self.filter.set_params(params.filter_cutoff, params.filter_resonance);
            for sample in frame.iter_mut() {
                *sample = self.filter.process(*sample, sample_rate);
            }
        }

        if params.chorus_enabled {
            self.chorus.process(frame, &params.chorus, sample_rate);
        }

        apply_gain_db(frame, params.gain_out_db);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SynthFxParams {
    pub chorus: ChorusParams,
    pub chorus_enabled: bool,
    pub gain_db: f32,
}
