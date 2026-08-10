# Modulus — Module Engine

The module engine is a pluggable DSP layer in `modulus-core::modules`.
Anything implementable as an `AudioModule` can be chained in a `ModuleGraph`
and driven per-frame with events.

Three ways to get a module:

1. **Native Rust module** — implement the `AudioModule` trait, register a
   builder in a `ModuleRegistry`. This is the “as easy as the old repos”
   path: each new unit is one struct plus one `register()` call.
2. **Lua patch** — a `.lua` script describes an ordered list of modules;
   compiled at load time into a native graph (see `docs/LUA.md`).
3. **Compiled module** — any language with a C ABI (Rust, C, C++, Python via
   ctypes/C extensions) exports the `modulus_module_*` ABI and is loaded at
   runtime with `DynamicModule` (see below).

## The `AudioModule` trait

```rust
pub trait AudioModule: Send {
    fn kind(&self) -> ModuleKind;            // Oscillator | Filter | Envelope | Effect
    fn name(&self) -> &str;                  // instance id
    fn rename(&mut self, id: &str) {}        // patch compiler support
    fn prepare(&mut self, sample_rate: f32); // may allocate, not RT
    fn reset(&mut self);
    fn note_on(&mut self, note: u8, velocity: f32, tuning_hz: f32) {}
    fn note_off(&mut self, note: u8) {}
    fn params(&self) -> &[ModuleParamSpec];  // name + default per param
    fn set_param(&mut self, name: &str, value: f32) -> bool;
    fn param_value(&self, name: &str) -> Option<f32>;
    fn process(&mut self, frame: &mut [f32; 2], events: &ModuleEvents, sample_rate: f32);
}
```

`ModuleGraph::process_frame` treats `ModuleKind::Oscillator` as a **source**
(its output is added into the frame) and everything else as a processor
(frame modified in place), in the order the modules were added.

## Built-in modules

| Name | Kind | Params |
| ---- | ---- | ------ |
| `oscillator` / `oscillator2` | Oscillator | `waveform` (0–7), `level`, `pitch_semitones` |
| `filter` | Filter | `filter_type` (0–3), `cutoff`, `resonance`, `smoothing_ms` |
| `envelope` | Envelope | `attack`, `decay`, `sustain`, `release` |
| `chorus` | Effect | `dry_wet`, `depth`, `rate`, `voices`, `delay_ms`, `width` |
| `gain` | Effect | `gain_db` |

All values are floats by design (Lua/ABI friendly).

## Adding a native module

```rust
use modulus_core::modules::*;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct MyFlangerParams {
    pub depth: f32,
}

pub struct FlangerModule { /* state */ }
impl AudioModule for FlangerModule { /* ... */ }

fn register(registry: &mut ModuleRegistry) {
    registry.register(
        "flanger",
        ModuleKind::Effect,
        Arc::new(|| -> Box<dyn AudioModule> { Box::new(FlangerModule::new()) }),
    );
}
```

That's the whole integration — Lua patches and hosts can now address
`flanger`.

## Compiled modules (plugin host)

`modulus-core::abi` defines the C ABI. A shared library must export:

```c
const ModulusModuleInfo* modulus_module_info(void);
void* modulus_module_create(void);
void  modulus_module_destroy(void* module);
void  modulus_module_prepare(void* module, float sample_rate);
void  modulus_module_reset(void* module);
void  modulus_module_process(void* module, float* in_l, float* in_r,
                             const float* params, float sample_rate);
```

`ModulusModuleInfo` carries a FourCC magic (`MODU`), API version (1), a
module kind constant, `param_count`, name, parameter names and defaults. See
`crates/demo-module/src/lib.rs` for a complete Rust implementation (it also
shows the recommended `catch_unwind` guard and `# Safety` contracts).

Loading:

```rust
// SAFETY: the library must implement the ABI correctly.
let module = unsafe {
    modulus_core::modules::host::DynamicModule::open(Path::new("my_module.dll"))?
};
```

`DynamicModule` implements `AudioModule`, so it slots into any `ModuleGraph`
next to native modules, giving you:

- **Rust** — export the ABI directly (see demo-module).
- **C / C++** — same ABI.
- **Python** — compile a C extension implementing the ABI, or use
  ctypes to expose the six symbols from a DLL built with CFFI/Cython.
- No hot-swap while rendering: libraries are loaded at setup time; unloading
  mid-session is intentionally unsupported.

## Real-time rules for module authors

- `process` must not allocate, lock, or call into the OS. Same rules as the
  rest of modulus-core.
- `prepare` may allocate
  (delay lines, lookups) and is called from `initialize`/`set_sample_rate`.
- Exported ABI functions must not unwind: wrap bodies in `catch_unwind`
  (see demo-module) or compile with `panic = "abort"`.