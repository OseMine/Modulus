//! Modulus Core — the shared, real-time-safe DSP library behind both
//! Modulus (synthesizer) and Modulus FX (effect processor).
//!
//! Every `process` path in this crate is allocation-free and lock-free:
//! no heap allocation, no locks, no blocking calls may appear in the audio
//! callback. The only allocations in this crate happen in `set_sample_rate`,
//! which is intended to be called from the plugin's `initialize()` hook.

pub mod abi;
pub mod envelope;
pub mod filter;
pub mod fx;
pub mod midi;
pub mod modules;
pub mod oscillator;
pub mod rng;
pub mod synth;
pub mod voice;
pub mod waveform;

pub use envelope::Adsr;
pub use filter::{FilterType, OnePoleSmoother, VariableFilter};
pub use fx::{Chorus, ChorusParams, FxEngine, FxFrameParams, SynthFxParams};
pub use midi::midi_note_to_freq;
pub use oscillator::Oscillator;
pub use rng::FastRng;
pub use synth::SynthEngine;
pub use voice::{Osc2Mode, SynthFrameParams, Voice, VoicePool, MAX_VOICES};
pub use waveform::Waveform;
