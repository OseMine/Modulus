# Modulus Konsolidierung — Archiv erledigter Aufgaben

Alle unter `todo.md` abgehakten Aufgaben wurden geprüft und als erledigt
archiviert. Stand: 2026-08-11.

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

## Phase 6b: Module Categories + Am-Synth Bridge — DONE
- [x] `ModuleKind` split into categories: `SoundGen`, `Envelope`, `Modulator`, `Fx` (+ `is_source`, `label`)
- [x] `native/` folder mirrors categories: `soundgen/`, `envelope/`, `modulator/`, `fx/`
- [x] `registry.names_by_kind()` to enumerate a category (GUI palettes, docs)
- [x] `am_bridge` sound generator added: `Am-Synth` carrier/modulator bridge with `Mix`/`AM` modes + `am_depth` (reuses the voice `Osc2Mode` architecture)
- [x] `lfo` modulator added (free-running, depth 0..1, tremolo-ready)
- [x] ABI kind constants renumbered to match (`SOUNDGEN=0`, `ENVELOPE=1`, `MODULATOR=2`, `FX=3`); host + demo-module + tests updated
- [x] New tests: 9 category/bridge/LFO + 1 Lua bridge patch (16 total, all passing)
- [x] `example_patch.lua` now showcases am_bridge + lfo; render peak 0.206, non-silent
- [x] Docs updated (MODULES.md table + categories, LUA.md semantics, ARCHITECTURE.md)

## Phase 7: Documentation & Delivery — DONE
- [x] `README.md` + `docs/{ARCHITECTURE,MIGRATION,PARAMETERS,BUILDING,MODULES,LUA}.md`

## Phase 6c: Synth Setup Configs — DONE
- [x] `SynthSetup` (feature `setup`): JSON configs defining synth topology/routing (`setups/`)
- [x] `SynthGraph::process_frame` fixed routing: sources → bus → filter (+pregain) → amp env → modulator (to_amp/to_filter_octaves), `cv()`-driven cutoff/anp modulation
- [x] Optional fixed `model` per slot; role defaults (`oscillator`/`filter`/`envelope`/`lfo`), category validated at build
- [x] `AudioModule::cv()` default + `adsr`/`lfo` implementations
- [x] `setups/default.json` + `setups/bridge_lead.json`; user setups dir (`%APPDATA%/Modulus/setups`)
- [x] 13 tests (`tests/synth_setup.rs`) + `setup_player` example renders to WAV
- [x] `docs/SETUPS.md` + README feature bullet

## Phase 8: GitOps — DONE
- [x] `new` branch merged into `main` (Cargo.lock conflict resolved)
- [x] `.github/actions/setup` (toolchain+rust-cache), `.github/actions/checks`, `.github/actions/bundle` composite actions
- [x] `build.yml`: 3-OS matrix running fmt/clippy/tests/bundle via the actions
- [x] `release.yml`: tag-triggered, bundles + GitHub Release with OpenCode-generated notes
- [x] `opencode.yml`: hardened (concurrency, write-permission check, timeout, bot guard) + `rust-check` job
- [x] All changes committed + pushed to `origin/main`

## Review-Runde 2026-08-11 (2. Review: GH Actions, Bridge-Engines-Review, FX) — DONE

Berichte: `reports/review-2026-08-11-gh-actions.md`, `reports/review-2026-08-11.md`.

### GitHub Actions: Bugs (G1–G8)
- [x] G1: `setup/action.yml` installiert `libx11-dev` + `libx11-xcb-dev`; doppelter Install-Schritt aus `checks/action.yml` entfernt
- [x] G2: `/oc`-Handler erhält endlich `prompt:`-Input (`${{ github.event.comment.body }}`)
- [x] G3: `concurrency`-Guard (`cancel-in-progress: true`) in `opencode-review.yml` + `opencode-todo-issues.yml`
- [x] G4: `release.yml` baut den Windows-Installer im `rust`-Job mit und lädt ihn ins Release (`Modulus-Installer-*.exe`)
- [x] G5: Release-Notes-Fallback `git log --oneline -60 HEAD` bei fehlendem `PREV_TAG` (beide Zweige)
- [x] G6: opencode-Job mit `if: always() && (…)` — `/oc` läuft auch bei rotem `rust-check`
- [x] G7: `actions/checkout@v6` → `@v4` in beiden opencode-Workflows (Versionseinheit + Node-20-Deprecation)
- [x] G8: `build.yml` PR-Trigger mit `paths-ignore` (docs/**, reports/**, setups/**, *.md, .github/**)

### Übernommen aus PR #7 (Bridge-Engines-Review, B1–B9)
- [x] B1: `filter`-Modul ruft `set_type()` auf (Feld wurde nie an die DSP-Unit durchgereicht)
- [x] B2: `analog_saw` clammt auf ±1 (war −3.0..1.0)
- [x] B3: `pregain_db` wirkt als Filter-Drive (vor dem Filter), nicht als zweiter Master-Level; `amp_mod` auf 0..1 geklemmt
- [x] B4: Sources free-runnen nach `note_off` (Gate-Felder entfernt) — Release-Tail des Amp-Envelopes hörbar
- [x] B5: Velocity ins `envelope`-Modul (note_on-Velocity skaliert die Amplitude; `cv()` bleibt velocity-unabhängig)
- [x] B6: `Le13700` scale 0.7 — klar vom Roland (1.0) unterscheidbar
- [x] B7: `oscillator` + `am_bridge`/`fm_bridge` clammen `level` 0..1 und `pitch_semitones` ±24
- [x] B8: LFO-CV zentriert um 1 (`1 ± depth`) — Filter-Modulation symmetrisch um die Basis (SETUPS.md korrigiert)
- [x] B9: `ModuleEvents.note_on/note_off`-Felder entfernt (6 Konstruktionsstellen aktualisiert)

### FX-Review-Befunde
- [x] ARP-4075: `process_arp` als echte Feedback-Kaskade mit Stage-Clamp ±1 — Totbereich bei `res=1.0` behoben
- [x] Roland/LE13700 nicht mehr bitidentisch (siehe B6)
- [x] Chorus-`width` korrigiert: `channel_offset = π · (1 − width)` — width 0 = mono, 1 = antiphase
- [x] Chorus schreibt die Delay-Leitung bei `voices==0`/`dry_wet==0` weiter (kein Einfrieren)
- [x] `filt_smoothing`: `2π` aus der Koeffizienten-Formel entfernt (FX + Filter-Modul) — 50 ms ≈ τ=50 ms
- [x] FX: `filt_cutoff` mit `Logarithmic(30)`-, `filt_resonance` mit `Linear(50)`-Smoother; `smoothed.next()` pro Sample
- [x] Filter-Envelope bipolar: `filt_env_amount` Range −1..=1 (`modulus-synth`), `Linear(50)`-Smoother (#10)
- [x] Setup-Filter-Smoothing: Modul-Default 0 → 15 ms (#11)
- [x] Editor: `ctx.request_repaint()` pro Frame; `editor_state.is_open()` aus der Sample-Schleife gehoben (#12)
- [x] Chorus-Voices: Range 1..=8 im Modul + beiden Plugins (Bypass nur noch über die Enable-Params) (#13)
- [x] UI-Helfer geteilt: neues `crates/modulus-ui` (ACCENT/BG/PANEL, `dark_visuals()`, `slider_row()`, `section()`); Synth- + FX-Editor nutzen es (#14)
- [x] Review-Workflow von Koalitions-O-Mat auf Modulus umgestellt (`crates/`, `docs/`, `.github/`) (#9)

### Tests
- [x] `synth_setup`: `amp_modulation_changes_output` auf Leistung (Power) umgestellt; `note_off_releases_the_envelopes` prüft hörbaren Release-Tail + Stille nach ~1 s
- [x] `am_bridge`/`fm_bridge`-Gate-Tests → free-running („does_not_gate_itself")
- [x] 33 Tests grün; `clippy -D warnings` + `fmt --check` sauber

## Open Questions / Notes
- `Am-Synth` filter banks were never wired into its audio path; consolidated as single per-voice filter
- `variable-effects` chorus placeholder replaced with a real multi-tap modulated delay-line chorus
- `Am-Synth` MIDI now processed sample-accurate
- `velocity` already normalized `f32` at this nih-plug rev
- `editor_state.is_open()` skips expensive GUI-only work while window is closed
- Windows `linker_messages` warning during release bundle is benign MSVC export-lib noise