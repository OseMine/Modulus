//! Sound generator modules (`ModuleKind::SoundGen`): signal sources whose
//! output is added into the frame by the graph.

pub mod am_bridge;
pub mod oscillator;

use super::super::registry::ModuleRegistry;

/// Register every sound generator on the registry.
pub fn register(registry: &mut ModuleRegistry) {
    oscillator::register(registry);
    am_bridge::register(registry);
}
