# Modulus — Lua Patch Format

Lua patches compile to a native `ModuleGraph`; the Lua VM only runs at load
time, so the render path keeps the usual real-time guarantees.

## Example

```lua
-- scripts/lua/example_patch.lua
return {
  name = "Deep Pad",
  modules = {
    { kind = "oscillator", id = "osc1", waveform = 4, level = 0.5, pitch_semitones = 0 },
    { kind = "oscillator", id = "osc2", waveform = 0, level = 0.2, pitch_semitones = 7 },
    { kind = "filter",     id = "filt", filter_type = 0, cutoff = 2000, resonance = 0.3 },
    { kind = "envelope",   id = "env",  attack = 0.05, decay = 0.2, sustain = 0.6,
                                        release = 0.4 },
    { kind = "chorus",     id = "ch",   dry_wet = 0.3, depth = 0.4, rate = 1.0,
                                        voices = 3, delay_ms = 12, width = 0.5 },
    { kind = "gain",       id = "out",  gain_db = -6 },
  },
}
```

## Grammar

The chunk must return one **table** with at least:

- `modules`: an array of module **entry tables**, in signal-chain order.

Optional:

- `name`: string, informational.
- An entry may set `id` — the instance name used to address it later via
  `ModuleGraph::set_param(module, param, value)`. Default id = `kind`.

Each entry table:

- `kind` (string, required): a registered module name. Built-ins:
  `oscillator`, `oscillator2`, `filter`, `envelope`, `chorus`, `gain`.
- Everything else is a parameter assignment; values may be numbers or
  numeric strings. Unknown parameter names are ignored.

## Semantics

- **Order matters.** Oscillators are sources (output added into the frame);
  all other modules process the frame in place, in listed order.
- **Events.** `graph.note_on(note, velocity, tuning)` / `note_off(note)`
  are forwarded to every module (oscillators set frequency + gate,
  envelopes trigger/release).
- **Param types are floats.** `waveform` takes an index (0–7),
  `filter_type` an index (0–3); see `docs/PARAMETERS.md` for the order.
- **Errors.** Unknown `kind` fails the build (`UnknownModule`); syntax
  errors and non-table returns fail with `Lua(...)`.

## Compiling

```rust
use modulus_core::modules::{builtin_registry, lua, ModuleEvents, ModuleGraph};

let registry = builtin_registry();
let mut graph: ModuleGraph = lua::build_patch(&registry, source)?;
graph.prepare(44_100.0);

let mut frame = [0.0; 2];
let events = ModuleEvents { note_on: Some((60, 1.0)), note_off: None,
                            time_secs: 0.0, tuning_hz: 440.0 };
graph.process_frame(&mut frame, &events, 44_100.0);
```

Or offline:

```bash
cargo run -p modulus-core --example patch_player
```

writes `target/patch_output.wav` (set `MODULUS_PATCH`/`MODULUS_OUTPUT` for
custom paths).

## Extending

Any native module registered in the registry is available to Lua patches
automatically — no Lua code changes needed. See `docs/MODULES.md` for
registering new modules.