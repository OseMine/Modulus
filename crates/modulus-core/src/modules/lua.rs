//! Lua patch engine (feature `lua`).
//!
//! A patch is a Lua chunk that returns a table describing an ordered list of
//! modules:
//!
//! ```lua
//! return {
//!   name = "My Patch",
//!   modules = {
//!     { kind = "oscillator", id = "osc1", waveform = 4, level = 0.7 },
//!     { kind = "filter", id = "filt", filter_type = 0, cutoff = 1200, resonance = 0.3 },
//!     { kind = "envelope", id = "env", attack = 0.05, decay = 0.2, sustain = 0.6, release = 0.4 },
//!     { kind = "gain", id = "out", gain_db = -3 },
//!   },
//! }
//! ```
//!
//! The script runs at load time only; the result is a plain native
//! [`ModuleGraph`] of registered modules, so the render path stays
//! allocation-free and lock-free.

use mlua::Lua;

use super::registry::ModuleRegistry;
use super::{AudioModule, ModuleError, ModuleGraph};

/// Compile a Lua patch script into a native module graph.
pub fn build_patch(registry: &ModuleRegistry, source: &str) -> Result<ModuleGraph, ModuleError> {
    let lua = Lua::new();
    let value = lua
        .load(source)
        .set_name("modulus-patch")
        .eval::<mlua::Value>()
        .map_err(|err| ModuleError::Lua(err.to_string()))?;

    let patch = match value {
        mlua::Value::Table(table) => table,
        _ => {
            return Err(ModuleError::Lua(
                "patch must return a table".to_string(),
            ))
        }
    };

    let modules_value = patch
        .get::<mlua::Value>("modules")
        .map_err(|err| ModuleError::Lua(err.to_string()))?;
    let modules_table = match modules_value {
        mlua::Value::Table(table) => table,
        _ => {
            return Err(ModuleError::Lua(
                "patch table must contain a `modules` sequence".to_string(),
            ))
        }
    };

    let mut graph_modules: Vec<Box<dyn AudioModule>> = Vec::new();
    for entry in modules_table.sequence_values::<mlua::Table>() {
        let entry = entry.map_err(|err| ModuleError::Lua(err.to_string()))?;
        let module_name = entry
            .get::<String>("kind")
            .map_err(|err| ModuleError::Lua(err.to_string()))?;
        let instance_id: String = entry
            .get::<Option<String>>("id")
            .map_err(|err| ModuleError::Lua(err.to_string()))?
            .unwrap_or_else(|| module_name.clone());

        let mut module = registry.create(&module_name)?;
        let module_id = instance_id.clone();
        module.rename(module_id.as_str());

        for pair in entry.pairs::<mlua::Value, mlua::Value>() {
            let (key, value) = pair.map_err(|err| ModuleError::Lua(err.to_string()))?;
            let key_str = match key {
                mlua::Value::String(s) => s
                    .to_str()
                    .map_err(|err| ModuleError::Lua(err.to_string()))?
                    .to_string(),
                _ => continue,
            };
            if key_str == "kind" || key_str == "id" {
                continue;
            }
            if let Some(param_value) = value_to_f32(value) {
                module.set_param(&key_str, param_value);
            }
        }

        graph_modules.push(module);
    }

    Ok(ModuleGraph::new(graph_modules))
}

fn value_to_f32(value: mlua::Value) -> Option<f32> {
    match value {
        mlua::Value::Number(n) => Some(n as f32),
        mlua::Value::Integer(i) => Some(i as f32),
        mlua::Value::String(s) => s.to_str().ok().and_then(|s| s.parse().ok()),
        _ => None,
    }
}