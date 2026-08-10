//! Built-in native modules wrapping the existing Modulus DSP units.

mod chorus;
mod envelope;
mod filter;
mod gain;
mod oscillator;

use super::registry::ModuleRegistry;

/// Register every built-in module on the registry.
pub fn register_builtin_modules(registry: &mut ModuleRegistry) {
    oscillator::register(registry);
    filter::register(registry);
    envelope::register(registry);
    chorus::register(registry);
    gain::register(registry);
}