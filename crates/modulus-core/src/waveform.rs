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

    shaped * 2.0 - 1.0
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
    (bandlimited + DC_OFFSET) / (1.0 + DC_OFFSET.abs())
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

    sample * 4.0 / PI
}

fn vintage_saw(phase: f32) -> f32 {
    let t = 1.0 / phase.max(0.01);
    let b = 5.0 * phase.max(0.01);
    let a = -2.0 / (E.powf(-b * t) - 1.0);
    a * E.powf(-b * phase) + (1.0 - a)
}
