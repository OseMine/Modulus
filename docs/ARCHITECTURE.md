# Modulus — Architecture

This document describes the workspace layout, the module graph data flow and
the real-time guarantees of the codebase.

## Crate layout

```
C:\coding\modulus
├── Cargo.toml                 workspace: members + pinned nih-plug + patches
├── crates/modulus-core        shared DSP + module engine (no nih_plug dep)
│   └── src/
│       ├── waveform.rs        8 waveform generators (polyblep/VA variants)
│       ├── oscillator.rs      phase-accumulator, per-waveform RNG seed
│       ├── rng.rs             xorshift32 (replaces rand::thread_rng)
│       ├── envelope.rs        linear ADSR with corrected release
│       ├── filter.rs          VariableFilter (4 models), OnePoleSmoother
│       ├── fx.rs              Chorus (ring-buffer delay lines), FxEngine, gain
│       ├── voice.rs           Voice, VoicePool (8 voices, round-robin steal)
│       ├── synth.rs           SynthEngine (pool -> chorus -> gain)
│       ├── midi.rs            note/frequency conversion
│       ├── abi.rs             C ABI for compiled modules
│       └── modules/           module engine
│           ├── mod.rs         AudioModule trait, ModuleGraph, errors
│           ├── registry.rs    named module builders
│           ├── native/        built-in osc/filter/env/chorus/gain modules
│           ├── lua.rs         Lua patch compiler (feature `lua`)
│           └── host/          dynamic loader (feature `plugin-host`)
├── crates/modulus-synth       Modulus plugin (VST3/CLAP + egui editor)
├── crates/modulus-fx          Modulus FX plugin (VST3/CLAP + egui editor)
├── crates/demo-module         example compiled module (C ABI oscillator)
├── xtask                      bundle command (`cargo run -p xtask --release bundle`)
├── scripts                    per-platform build scripts
└── vendor/anymap              vendored fix for anymap 1.0.0-beta.2 (E0804)
```

`modulus-core` intentionally has **no nih-plug dependency**, so the DSP can be
unit-tested and reused without a plugin host.

## Pinned toolchain

- rustc/cargo 1.97.x (the pinned nih-plug rev `dfafe903` requires recent
  rustc; earlier versions break on `anymap 1.0.0-beta.2`, see below)
- `nih_plug` pinned by git rev in the root `Cargo.toml` — do **not** update
  this crate without a full API migration pass (the code is written against
  this exact rev: `velocity` is already normalized, `iter_samples()` yields
  channel iterators, `editor_state` persistence, etc.)
- `[patch.crates-io] anymap` points at `vendor/anymap`, a copy of
  1.0.0-beta.2 with the trait-object cast fixed via `std::mem::transmute`.

## Audio data flow

```
        MIDI events (sample-accurate)
        [voice pool: 8 voices, stealing]
        └─ per-voice: osc1 + osc2(AM bridge) -> env amp -> filter -> fenv
        pool output (mono -> stereo)
        └─ chorus (shared, stereo)
        └─ output gain (dB)
```

Modulus FX:

```
input -> gain in -> VariableFilter (4 models) -> Chorus -> gain out -> output
```

Both plugins run one `process()` pass over `buffer.iter_samples()` with all
DSP parameter reads coming from smoothed params (`smoothed.next()` for
continuous params). Nothing in the audio callback allocates or locks; the
`assert_process_allocs` nih-plug feature is enabled and panics if allocations
are attempted inside `process()`.

## GUI (egui)

`editor.rs` in each plugin crate builds an `nih_plug_egui` editor:

- `EguiState` is stored on the params struct behind
  `#[persist = "editor-state"]`, so the window size survives state restores.
- A `DesignState`/`Arc<Atomic*>` bridge shares live values (voice count for
  the synth) with the UI; writes happen only while `editor_state.is_open()`.
- Layout: `TopBottomPanel` header + `CentralPanel` scrolling sections with
  `ParamSlider::for_param` rows (grid of label + slider).
- A custom dark `Visuals` palette is applied in the editor's build closure.

## Module engine

See [MODULES](MODULES.md) and [LUA](LUA.md) for the full API.

- `AudioModule` trait: `kind/name/params/prepare/reset/note_on/note_off/
  set_param/process`.
- `ModuleKind` categories: `SoundGen` (sources, output added into the
  frame), `Envelope` + `Modulator` (gain curves multiplied into the frame,
  note-gated vs free-running), `Fx` (frame modified in place). The
  `native/` folder mirrors this: `soundgen/`, `envelope/`, `modulator/`,
  `fx/`, and the C ABI kind constants map 1:1 to it.
- Built-ins are thin wrappers around the existing DSP units; adding a new
  module is registering one builder (mirrors how the old `workspace/variable-*`
  repos each owned one DSP unit).
- `am_bridge` is the `Am-Synth` carrier/modulator bridge (Mix or AM modes,
  with `am_depth`) brought back as a sound generator module; `fm_bridge` is a
  classic DX7-style 2-operator FM pair (modulator phase-modulates the carrier).
- Lua patches are compiled at load time into a native graph; the Lua VM is
  never active in the render path.
- Compiled modules implement the `modulus_module_*` C ABI and are loaded via
  `DynamicModule` (libloading) — Rust, C, C++ or Python (via C extension) can
  be used; see `crates/demo-module` for a reference implementation.

## Real-time rules (enforced)

1. No `Vec::new`/`Box`/`String` etc. inside `process()` (panic under
   `assert_process_allocs`).
2. No locks/`Mutex`/blocking I/O in `process()`; `Atomic` ops for
   GUI<->audio handoff only.
3. `set_sample_rate`/`prepare` may allocate (called from `initialize`).
4. GUI state writes are guarded by `editor_state.is_open()` so hosts without
   an open editor never pay the cost.