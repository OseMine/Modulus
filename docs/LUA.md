# Modulus — Lua Patch Format

Lua patches compile to a native `ModuleGraph`; the Lua VM only runs at load
time, so the render path keeps the usual real-time guarantees.

## Example

```lua
-- scripts/lua/example_patch.lua
return {
  name = "Deep Pad",
  modules = {
    { kind = "am_bridge", id = "bridge",
      carrier_waveform = 4, carrier_level = 0.5, carrier_pitch = 0,
      modulator_waveform = 0, modulator_level = 0.5, modulator_pitch = 7,
      mode = 1, am_depth = 0.5 },
    { kind = "lfo",       id = "trem", waveform = 0, rate_hz = 4, depth = 0.2 },
    { kind = "envelope",  id = "env",  attack = 0.05, decay = 0.2, sustain = 0.6,
                                        release = 0.4 },
    { kind = "filter",    id = "filt", filter_type = 0, cutoff = 2000, resonance = 0.3 },
    { kind = "chorus",    id = "ch",   dry_wet = 0.3, depth = 0.4, rate = 1.0,
                                        voices = 3, delay_ms = 12, width = 0.5 },
    { kind = "gain",      id = "out",  gain_db = -6 },
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

- `kind` (string, required): a registered module name. Built-ins by
  category: **sound generators** — `oscillator`, `oscillator2`,
  `am_bridge`; **envelope** — `envelope`; **modulator** — `lfo`;
  **FX** — `filter`, `chorus`, `gain`.
- Everything else is a parameter assignment; values may be numbers or
  numeric strings. Unknown parameter names are ignored.

## Semantics

- **Order matters.** Sound generators are sources (output added into the
  frame); envelopes and modulators multiply gain into the frame; FX modules
  process the frame in place, in listed order.
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