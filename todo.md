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
- [x] `xtask` bundling helper
- [x] Vendored `anymap` patch (upstream beta.2 broken with rustc 1.97, E0804)

## Phase 3: Code Extraction & Refactoring — DONE
### 3a Oscillators (variable-synth + Am-Synth) — DONE
- [x] `Waveform` enum with all 8 waveform generators (Sine, Saw, Square, AnalogSaw, VASaw, AnalogSquare, VASquare, VintageSaw)
- [x] `Oscillator` phase-accumulator (unified sine osc + phase loop)
- [x] `FastRng` xorshift32 replacing RT-unsafe `rand::thread_rng()`
- [x] `Voice` / `VoicePool` (8-voice, round-robin stealing, AM bridge from Am-Synth)

### 3b Filter (variable-filter) — DONE
- [x] `VariableFilter` with 4 models (Moog, Roland, LE13700, ARP 4075), zero-alloc
- [x] `OnePoleSmoother` replacing `static mut` globals
- [x] Ladder filters unified into one implementation with per-model scale factor

### 3c Effects (variable-effects) — DONE
- [x] `Chorus` (multi-voice modulated delay, ring buffers, stereo width)
- [x] `Gain` (dB), `FxEngine` serial rack: in-gain -> filter -> chorus -> out-gain
- [x] Removed `Vec` allocation + `Box<dyn Effect>` from process path

### 3d Params — DONE
- [x] Flat prefixed IDs: `osc1_`, `osc2_`, `filt_`, `env_`, `fenv_`, `fx_`, `global_`
- [x] `modulus-synth`: 29 params; `modulus-fx`: 14 params
- [x] Param enum mapping (`ParamWaveform` -> core `Waveform`, etc.)

## Phase 4: Build & Bundling — PARTIALLY DONE
- [x] Release build passes for both plugins (rustc 1.97.1)
- [x] `cargo clippy` clean for all workspace crates
- [x] `cargo run -p xtask --release bundle` produces VST3 + CLAP bundles
- [ ] Runtime host verification (load in a DAW / plugin host to confirm `assert_process_allocs`)
- [ ] **GUI: modern egui editor for Modulus (synth)** — user requested
- [ ] **GUI: modern egui editor for Modulus FX** — user requested

## Phase 5: Documentation & Delivery — PENDING
- [ ] `README.md` at workspace root
- [ ] `docs/ARCHITECTURE.md` — workspace/crate structure
- [ ] `docs/MIGRATION.md` — how the 4 repos were merged (per-module mapping)
- [ ] `docs/PARAMETERS.md` — full parameter ID mapping tables
- [ ] `docs/BUILDING.md` — terminal commands to compile + bundle VST3/CLAP
- [ ] Final delivery report (chat summary)

## Phase 6: GitOps — PENDING
- [ ] `build.yml` GitHub Actions workflow (CI: fmt, clippy, release build, bundle)
- [ ] `release.yml` GitHub Actions workflow (tag-triggered; uses OpenCode via `anomalyco/opencode/github@latest`; bundles + uploads both plugins to a GitHub Release)
- [ ] `opencode.yml` workflow already present (issue/PR comments via `/oc` or `/opencode`)
- [ ] Commit all changes + push to `origin/main` (only when user confirms)

## CURRENT STATE — IN PROGRESS (work resumed Aug 5, 2026)
### GUI: egui editors — IN PROGRESS
- [x] Add `nih_plug_egui` (git = same pinned nih-plug rev) to `[workspace.dependencies]`
- [x] Add `nih_plug_egui` dep to `crates/modulus-synth/Cargo.toml` and `crates/modulus-fx/Cargo.toml`
- [x] Add `editor_state: Arc<EguiState>` with `#[persist = "editor-state"]` to `ModulusParams` (synth, 640x520)
- [x] Add `editor_state: Arc<EguiState>` with `#[persist = "editor-state"]` to `ModulusFxParams` (fx, 520x480)
- [ ] Write `editor.rs` for Modulus (synth): header, collapsible sections (Oscillators, Filter, Amp Env, Filter Env, Chorus, Output), `ParamSlider::for_param` rows
- [ ] Write `editor.rs` for Modulus FX: header, sections (Filter, Chorus, Gain In/Out), `ParamSlider::for_param` rows
- [ ] Wire `fn editor(...)` into both `Plugin` impls via `create_egui_editor(...)`
- [ ] Rebuild (`cargo build --release`), clippy, and re-bundle with xtask to verify

## Open Questions / Notes
- Original `Am-Synth` filter banks (carrier/modulator/global) were never wired into its audio path; consolidated as a single per-voice filter in Modulus
- `variable-effects` chorus was a placeholder (`(sample_rate * rate).sin()` modulation); replaced with a real multi-tap modulated delay-line chorus
- `Am-Synth` MIDI events were processed after audio (out of order); now sample-accurate in Modulus
- `velocity` is already normalized `f32` at this nih-plug rev (no `/ 127.0` needed)
- `editor_state.is_open()` is available to skip expensive GUI-only work while the window is closed
- `crates/modulus-core/src/fx.rs` still has an uncommitted `Default` impl for `Chorus`/`FxEngine` (clippy fix) that must be included in the commit
