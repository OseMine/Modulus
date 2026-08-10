//! Modulator modules (`ModuleKind::Modulator`): free-running modulation
//! sources that multiply a slowly varying signal into the frame.
//!
//! Unlike envelopes, modulators ignore note events and run continuously,
//! making them suitable for tremolo, vibrato-style FX and slow sweeps.

pub mod lfo;

use super::super::registry::ModuleRegistry;

/// Register every modulator module on the registry.
pub fn register(registry: &mut ModuleRegistry) {
    lfo::register(registry);
}