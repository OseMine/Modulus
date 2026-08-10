# Modulus Consolidation — Task List

## Phase 1: Reconnaissance & Cloning — DONE
- [x] Clone `variable-synth`, `Am-Synth`, `variable-filter`, `variable-effects` into `workspace/`
- [x] Analyze Cargo.toml, module layout, DSP structs, params, GUI integrations
- [x] Note: all 4 repos pin the same nih-plug git rev (`dfafe90349aa3d8e40922ec031b6d673803d6432`)

## Phase 2: Workspace Architecture — DONE
- [x] Root workspace `Cargo.toml` with `crates/` layout
- [x] `modulus-core` shared DSP library (no nih_plug dependency)
- [x] `modulus-synth` plugin crate (Modulus)
- [x] `modulus-fx` plugin crate (Modulus FX)
- [x] `xtask` bundling helper (cross-platform host layouts)
- [x] Vendored `anymap` patch (upstream beta.2 broken with rustc 1.97, E0804)

## Phase 3: Code Extraction & Refactoring — DONE
### 3a Oscillators (variable-synth + Am-Synth) — DONE
- [x] `Waveform` enum with all 8 waveform generators
- [x] `Oscillator` phase-accumulator (unified sine osc + phase loop)
- [x] `FastRng` xorshift32 replacing RT-unsafe `rand::thread_rng()`
- [x] `Voice` / `VoicePool` (8-voice, round-robin stealing, AM bridge)

### 3b Filter (variable-filter) — DONE
- [x] `VariableFilter` with 4 models (Moog, Roland, LE13700, ARP 4075), zero-alloc
- [x] `OnePoleSmoother` replacing `static mut` globals
- [x] Ladder filters unified into one implementation with per-model scale factor

### 3c Effects (variable-effects) — DONE
- [x] `Chorus` (real modulated delay line, stereo width)
- [x] `Gain` (dB), `FxEngine` serial rack
- [x] Removed `Vec` allocation + `Box<dyn Effect>` from process path

### 3d Params — DONE
- [x] Flat prefixed IDs (`osc1_`, `osc2_`, `filt_`, `env_`, `fenv_`, `fx_`, `global_`)
- [x] `modulus-synth`: 29 params; `modulus-fx`: 14 params (+ editor state)

## Phase 4: Build & Bundling — DONE (DAW check deferred)
- [x] Release build passes for both plugins (rustc 1.97.1)
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] `cargo run -p xtask --release bundle` produces VST3 + CLAP bundles
- [x] `scripts/build.ps1` / `scripts/build.sh` run the full gate (fmt, clippy, tests, bundle)
- [ ] Runtime host verification in a DAW — deferred: no DAW on this machine (covered nightly by `--test` plugin_host)

## Phase 5: GUI Editors — DONE
- [x] `nih_plug_egui` at pinned rev in workspace deps; `editor_state: Arc<EguiState>` (`#[persist]`) on both param sets
- [x] `crates/modulus-synth/src/editor.rs`: header, collapsible sections (Oscillators, Filter, Amp Env, Filter Env, Chorus, Output), live voice-count meter
- [x] `crates/modulus-fx/src/editor.rs`: header, sections (Filter, Chorus, Gain In/Out)
- [x] `ParamSlider::for_param` rows, dark visuals, `create_egui_editor` wired into both `Plugin::editor()`

## Phase 6: Module Engine — DONE
- [x] Core engine: `AudioModule` trait + `ModuleGraph`, registry, native modules (3 osc types, filter, envelope, chorus, gain)
- [x] Lua engine: `mlua 0.12` (lua54) patch compiler loading scripts at patch time (zero-alloc in `process()`)
- [x] Plugin engine: stable C-ABI (`abi.rs`, magic `0x4D4F_4455`, API v1) + `DynamicModule` host via `libloading`
- [x] `demo-module` reference compiled module (cdylib, no-unwind, `catch_unwind` + panic flag)
- [x] Tests: 5 Lua + 1 plugin-host (live DLL load), all passing; `patch_player` example render verified
- [x] `docs/MODULES.md` + `docs/LUA.md` explain how to add modules / write Lua patches

## Phase 7: Documentation & Delivery — DONE
- [x] `README.md` + `docs/{ARCHITECTURE,MIGRATION,PARAMETERS,BUILDING,MODULES,LUA}.md`

## Phase 8: GitOps — DONE
- [x] `new` branch merged into `main` (Cargo.lock conflict resolved)
- [x] `.github/actions/setup` (toolchain+rust-cache), `.github/actions/checks`, `.github/actions/bundle` composite actions
- [x] `build.yml`: 3-OS matrix running fmt/clippy/tests/bundle via the actions
- [x] `release.yml`: tag-triggered, bundles + GitHub Release with OpenCode-generated notes
- [x] `opencode.yml`: hardened (concurrency, write-permission check, timeout, bot guard) + `rust-check` job
- [x] All changes committed + pushed to `origin/main`

## Open Questions / Notes
- `Am-Synth` filter banks were never wired into its audio path; consolidated as single per-voice filter
- `variable-effects` chorus placeholder replaced with a real multi-tap modulated delay-line chorus
- `Am-Synth` MIDI now processed sample-accurate
- `velocity` already normalized `f32` at this nih-plug rev
- `editor_state.is_open()` skips expensive GUI-only work while window is closed
- Windows `linker_messages` warning during release bundle is benign MSVC export-lib noise