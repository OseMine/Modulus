use std::f32::consts::{E, PI};

use crate::rng::FastRng;

const TWO_PI: f32 = 2.0 * PI;

/// The eight waveform generators consolidated from `variable-synth`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    AnalogSaw,
    VASaw,
    AnalogSquare,
    VASquare,
    VintageSaw,
}

impl Waveform {
    pub const ALL: [Waveform; 8] = [
        Waveform::Sine,
        Waveform::Saw,
        Waveform::Square,
        Waveform::AnalogSaw,
        Waveform::VASaw,
        Waveform::AnalogSquare,
        Waveform::VASquare,
        Waveform::VintageSaw,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Waveform::Sine => "Sine",
            Waveform::Saw => "Saw",
            Waveform::Square => "Square",
            Waveform::AnalogSaw => "Analog Saw",
            Waveform::VASaw => "VA Saw",
            Waveform::AnalogSquare => "Analog Square",
            Waveform::VASquare => "VA Square",
            Waveform::VintageSaw => "Vintage Saw",
        }
    }

    pub fn from_index(index: usize) -> Self {
        Waveform::ALL[index.min(Waveform::ALL.len() - 1)]
    }

    pub fn generate(self, phase: f32, rng: &mut FastRng) -> f32 {
        match self {
            Waveform::Sine => phase.sin(),
            Waveform::Saw => 2.0 * (phase / TWO_PI).rem_euclid(1.0) - 1.0,
            Waveform::Square => {
                if phase < PI {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::AnalogSaw => analog_saw(phase, rng),
            Waveform::VASaw => va_saw(phase, rng),
            Waveform::AnalogSquare => analog_square(phase),
            Waveform::VASquare => va_square(phase, rng),
            Waveform::VintageSaw => vintage_saw(phase),
        }
    }
}

fn analog_saw(phase: f32, rng: &mut FastRng) -> f32 {
    const SHARPNESS: f32 = 1.5;
    const ASYMMETRY: f32 = 0.6;
    const JITTER: f32 = 0.01;

    let normalized = (phase / TWO_PI).rem_euclid(1.0);
    let jittered = (normalized + (rng.f32_unit() - 0.5) * JITTER).rem_euclid(1.0);

    let asymmetric = if jittered < ASYMMETRY {
        jittered / ASYMMETRY
    } else {
        (jittered - ASYMMETRY) / (1.0 - ASYMMETRY) - 1.0
    };

    let shaped = if asymmetric >= 0.0 {
        asymmetric.powf(SHARPNESS)
    } else {
        -((-asymmetric).powf(SHARPNESS))
    };

    shaped.clamp(-1.0, 1.0)
}

fn va_saw(phase: f32, rng: &mut FastRng) -> f32 {
    const HARMONICS: usize = 10;
    const DC_OFFSET: f32 = 0.05;
    const JITTER: f32 = 0.001;

    let jittered = phase + (rng.f32_unit() - 0.5) * JITTER * TWO_PI;
    let p = jittered.rem_euclid(TWO_PI);

    let mut sample = 0.0;
    let mut n = 1;
    while n <= HARMONICS {
        let harmonic = n as f32;
        let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
        sample += sign * (p * harmonic).sin() / harmonic;
        n += 1;
    }

    let bandlimited = sample * 2.0 / PI;
    let output = (bandlimited + DC_OFFSET) / (1.0 + DC_OFFSET.abs());
    // The band-limited partial sum can overshoot the unit norm; clamp so the
    // waveform never exceeds [-1, 1] (was peaking at ~1.083).
    output.clamp(-1.0, 1.0)
}

fn analog_square(phase: f32) -> f32 {
    const TRANSITION: f32 = 0.1;

    let p = (phase / TWO_PI).rem_euclid(1.0);
    let width = TRANSITION / 2.0;

    if p < 0.5 - width {
        1.0
    } else if p < 0.5 + width {
        1.0 - (p - (0.5 - width)) / TRANSITION * 2.0
    } else if p < 1.0 - width {
        -1.0
    } else {
        -1.0 + (p - (1.0 - width)) / TRANSITION * 2.0
    }
}

fn va_square(phase: f32, rng: &mut FastRng) -> f32 {
    const HARMONICS: usize = 3;
    const JITTER: f32 = 0.001;

    let jittered = phase + (rng.f32_unit() - 0.5) * JITTER * TWO_PI;
    let p = jittered.rem_euclid(TWO_PI);

    let mut sample = 0.0;
    let mut k = 1;
    while k <= HARMONICS {
        let harmonic = (2 * k - 1) as f32;
        sample += (p * harmonic).sin() / harmonic;
        k += 1;
    }

    // Clamp: the 3-harmonic partial sum reaches ~1.19 at the harmonics'
    // phase alignment, which would clip earlier than the other forms.
    (sample * 4.0 / PI).clamp(-1.0, 1.0)
}

fn vintage_saw(phase: f32) -> f32 {
    let t = 1.0 / phase.max(0.01);
    let b = 5.0 * phase.max(0.01);
    let a = -2.0 / (E.powf(-b * t) - 1.0);
    // The exponential swing dips just below -1; clamp keeps the unit norm.
    (a * E.powf(-b * phase) + (1.0 - a)).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_waveforms_stay_within_unit_range() {
        let mut rng = FastRng::new(0x1234_5678);
        for waveform in Waveform::ALL {
            let mut min = f32::MAX;
            let mut max = f32::MIN;
            let mut phase = 0.0;
            // Sweep many frequencies/phases, including the band-limited
            // partial-sum alignments that used to overshoot.
            for step in 0..20_000 {
                phase = (phase + TWO_PI * (0.01 + 0.07 * (step % 9) as f32)).rem_euclid(TWO_PI);
                let sample = waveform.generate(phase, &mut rng);
                min = min.min(sample);
                max = max.max(sample);
            }
            assert!(min >= -1.0, "{waveform:?} undershoots: min {min}");
            assert!(max <= 1.0, "{waveform:?} overshoots: max {max}");
        }
    }
}
