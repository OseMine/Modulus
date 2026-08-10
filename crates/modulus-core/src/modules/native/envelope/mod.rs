//! Envelope modules (`ModuleKind::Envelope`): note-gated amplitude shapers
//! that multiply a gain curve into the frame.

pub mod adsr;

use super::super::registry::ModuleRegistry;

/// Register every envelope module on the registry.
pub fn register(registry: &mut ModuleRegistry) {
    adsr::register(registry);
}