//! Build:
//!
//! | Category | Folder | Kind |
//! | -------- | ------ | ---- |
//! | Sound generators (synth voices, oscillators, bridges) | `soundgen/` | `ModuleKind::SoundGen` |
//! | Note-gated amplitude shapers (ADSR, ...) | `envelope/` | `ModuleKind::Envelope` |
//! | Free-running modulators (LFO, ...) | `modulator/` | `ModuleKind::Modulator` |
//! | Audio processors (filter, chorus, gain, ...) | `fx/` | `ModuleKind::Fx` |

pub mod envelope;
pub mod fx;
pub mod modulator;
pub mod soundgen;

use super::registry::ModuleRegistry;

/// Register every built-in module on the registry.
pub fn register_builtin_modules(registry: &mut ModuleRegistry) {
    soundgen::register(registry);
    envelope::register(registry);
    modulator::register(registry);
    fx::register(registry);
}