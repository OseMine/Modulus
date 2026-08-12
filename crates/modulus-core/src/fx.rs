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
        if self.size == 0 {
            return;
        }

        let voices = params.voices.clamp(0, 8);
        let max_delay_samples = (MAX_DELAY_MS * 0.001 * sample_rate - 2.0).max(1.0);
        let base_delay = (params.delay_ms * 0.001 * sample_rate).clamp(0.0, max_delay_samples);
        let depth_samples = params.depth.clamp(0.0, 1.0) * max_delay_samples * 0.5;
        let lfo_step = TWO_PI * params.rate / sample_rate;
        // Width maps 0 (mono, right LFO in phase) to 1 (maximum stereo
        // spread, right LFO in anti-phase with the left one).
        let channel_offset = PI * (1.0 - params.width.clamp(0.0, 1.0));
        let voice_spread = if voices > 0 {
            TWO_PI / voices as f32
        } else {
            0.0
        };
        let dry = 1.0 - params.dry_wet;
        let wet_gain = if voices > 0 {
            params.dry_wet / voices as f32
        } else {
            1.0
        };
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

            if voices > 0 {
                let mut wet = 0.0;
                for voice in 0..voices {
                    let voice_phase = base_phase + voice as f32 * voice_spread;
                    let modulation = 0.5 + 0.5 * voice_phase.sin();
                    let delay =
                        (base_delay + depth_samples * modulation).clamp(1.0, max_delay_samples);
                    let delay_floor = delay.floor();
                    let fraction = delay - delay_floor;
                    let index = (*write + size - delay_floor as usize) & mask;
                    let next_index = (index + 1) & mask;
                    let delayed = buffer[index] * (1.0 - fraction) + buffer[next_index] * fraction;
                    wet += delayed;
                }
                wet *= wet_gain;

                let input = frame[channel];
                let buffer = if channel == 0 {
                    &mut self.buffer_left
                } else {
                    &mut self.buffer_right
                };
                frame[channel] = input * dry + wet;
                buffer[*write] = input;
                *write = (*write + 1) & mask;
            } else {
                // voices = 0 means bypass: pass the input through dry, but
                // keep the delay lines moving so re-enabling the chorus
                // does not replay stale audio.
                let input = frame[channel];
                let buffer = if channel == 0 {
                    &mut self.buffer_left
                } else {
                    &mut self.buffer_right
                };
                buffer[*write] = input;
                *write = (*write + 1) & mask;
            }
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

impl Default for Chorus {
    fn default() -> Self {
        Self::new()
    }
}

/// Serial effects rack for Modulus FX: input gain -> variable filter ->
/// chorus -> output gain.
pub struct FxEngine {
    filter_left: crate::filter::VariableFilter,
    filter_right: crate::filter::VariableFilter,
    chorus: Chorus,
}

impl FxEngine {
    pub fn new() -> Self {
        Self {
            filter_left: crate::filter::VariableFilter::new(),
            filter_right: crate::filter::VariableFilter::new(),
            chorus: Chorus::new(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.chorus.set_sample_rate(sample_rate);
    }

    pub fn reset(&mut self) {
        self.filter_left.reset();
        self.filter_right.reset();
        self.chorus.reset();
    }

    pub fn process(&mut self, frame: &mut [f32; 2], params: &FxFrameParams, sample_rate: f32) {
        apply_gain_db(frame, params.gain_in_db);

        if params.filter_enabled {
            // One filter state per channel: a single shared filter lets the
            // left channel's state bleed into the (silent) right channel.
            self.filter_left.set_type(params.filter_type);
            self.filter_left
                .set_smoothing(params.filter_smoothing_coeff);
            self.filter_left
                .set_params(params.filter_cutoff, params.filter_resonance);
            frame[0] = self.filter_left.process(frame[0], sample_rate);

            self.filter_right.set_type(params.filter_type);
            self.filter_right
                .set_smoothing(params.filter_smoothing_coeff);
            self.filter_right
                .set_params(params.filter_cutoff, params.filter_resonance);
            frame[1] = self.filter_right.process(frame[1], sample_rate);
        }

        // Always run the chorus so the delay lines keep moving while the
        // effect is bypassed; a dry mix still writes the current input into
        // the buffer. Skipping the call entirely used to freeze the delay
        // line and replay stale audio on re-enable.
        let mut chorus_params = params.chorus;
        if !params.chorus_enabled {
            chorus_params.dry_wet = 0.0;
        }
        self.chorus.process(frame, &chorus_params, sample_rate);

        apply_gain_db(frame, params.gain_out_db);
    }
}

impl Default for FxEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SynthFxParams {
    pub chorus: ChorusParams,
    pub chorus_enabled: bool,
    pub gain_db: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_params(chorus_enabled: bool) -> FxFrameParams {
        FxFrameParams {
            filter_type: FilterType::Moog,
            filter_cutoff: 500.0,
            filter_resonance: 0.0,
            filter_smoothing_coeff: 0.0,
            filter_enabled: true,
            gain_in_db: 0.0,
            chorus: ChorusParams {
                dry_wet: 1.0,
                depth: 1.0,
                rate: 0.5,
                voices: 2,
                delay_ms: 50.0,
                width: 1.0,
            },
            chorus_enabled,
            gain_out_db: 0.0,
        }
    }

    #[test]
    fn filter_does_not_couple_channels() {
        // Left channel gets a 1 kHz sine, right channel silence. A single
        // shared filter would ring the right channel with processed left
        // audio (right peak ~0.135); per-channel filters must keep it ~0.
        let mut engine = FxEngine::new();
        engine.set_sample_rate(44_100.0);
        let params = frame_params(false);

        let mut right_peak: f32 = 0.0;
        for i in 0..2048 {
            let left = (TWO_PI * 1000.0 * i as f32 / 44_100.0).sin() * 0.5;
            let mut frame = [left, 0.0];
            engine.process(&mut frame, &params, 44_100.0);
            right_peak = right_peak.max(frame[1].abs());
        }
        assert!(right_peak < 1e-3, "right channel leaked: peak {right_peak}");
    }

    #[test]
    fn chorus_bypass_keeps_delay_line_running() {
        let mut engine = FxEngine::new();
        engine.set_sample_rate(44_100.0);
        let mut params = frame_params(true);

        // Fill the delay lines with a loud signal.
        for _ in 0..44_100 {
            engine.process(&mut [1.0, 1.0], &params, 44_100.0);
        }

        // Bypass and feed silence for longer than the delay-line length so
        // any stale audio would have flushed out of a delay line that keeps
        // moving (100 ms @ 44.1 kHz = 4410 samples, buffer is larger).
        params.chorus_enabled = false;
        for _ in 0..20_000 {
            engine.process(&mut [0.0, 0.0], &params, 44_100.0);
        }

        // Re-enable: must stay silent instead of replaying stale audio.
        params.chorus_enabled = true;
        let mut peak: f32 = 0.0;
        for _ in 0..4410 {
            let mut frame = [0.0, 0.0];
            engine.process(&mut frame, &params, 44_100.0);
            peak = peak.max(frame[0].abs()).max(frame[1].abs());
        }
        assert!(
            peak < 1e-3,
            "stale delay replayed after bypass: peak {peak}"
        );
    }
}
