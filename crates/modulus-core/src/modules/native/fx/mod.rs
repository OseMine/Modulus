//! FX modules (`ModuleKind::Fx`): audio processors that shape the frame in
//! place (filter, chorus, gain, ...).

pub mod chorus;
pub mod filter;
pub mod gain;

use super::super::registry::ModuleRegistry;

/// Register every FX module on the registry.
pub fn register(registry: &mut ModuleRegistry) {
    filter::register(registry);
    chorus::register(registry);
    gain::register(registry);
}
