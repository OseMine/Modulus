# Modulus

Hybrid subtractive synthesizer **Modulus** and its companion **Modulus FX**
multi-effects processor, built as one Rust workspace on
[nih-plug](https://github.com/robbert-vdh/nih-plug) (VST3 + CLAP).

Both plugins ship with a modern [egui](https://github.com/emilk/egui) editor
and a real-time-safe DSP core that was consolidated from four older plugins
(`variable-synth`, `Am-Synth`, `variable-filter`, `variable-effects`).

## Plugins

| Plugin        | Name       | Format      | Description                            |
| ------------- | ---------- | ----------- | -------------------------------------- |
| `modulus-synth` | Modulus  | VST3, CLAP  | Dual-oscillator subtractive synth (8 voices) |
| `modulus-fx` | Modulus FX | VST3, CLAP  | Filter + chorus + gain insert effect   |

## Features

- **8 waveforms** including VA (band-limited) saw/square variants
- **4 filter models**: Moog, Roland, LE13700, ARP 4075 (unified ladder core)
- **AM second-oscillator mode** with adjustable depth
- Independent **amp and filter ADSR envelopes**
- **Chorus** (multi-voice modulated delay, stereo width, up to 8 voices)
- Sample-accurate MIDI automation, zero allocation in the audio callback
- **Module engine**: trait-based `AudioModule` API, Lua patch compiler, and a
  C-ABI plugin host for compiled third-party modules (Rust/C/C++/Python
  extensions)
- **Synth setup configs**: JSON files defining the synth topology and routing
  (4 soundgens → mixer → filter with pregain → amp/filter envelopes with
  contour → modulator with amp/filter amounts)
- egui editors with persisted window size

## Workspace layout

```
crates/
  modulus-core/    shared RT-safe DSP (also the module engine host)
  modulus-synth/   Modulus plugin (VST3/CLAP)          [+ egui editor]
  modulus-fx/      Modulus FX plugin (VST3/CLAP)        [+ egui editor]
  demo-module/     example compiled module (C ABI) for the plugin host
xtask/             bundling helper (VST3/CLAP bundles)
scripts/           per-platform build+bundle scripts
vendor/anymap/     vendored anymap patch (rustc 1.97 / pinned nih-plug)
docs/              architecture, migration, parameter, building docs
```

## Quick start

```bash
# required tools: Rust 1.97+, git

cargo build --release -p modulus-synth -p modulus-fx   # build both plugins
cargo run -p xtask --release bundle                     # create .vst3/.clap bundles
# bundles land in target/bundled/
```

Windows/macOS/Linux one-shot scripts:

```powershell
# Windows
.\scripts\build.ps1
```

```bash
# macOS / Linux
./scripts/build.sh
```

## Documentation

- [ARCHITECTURE](docs/ARCHITECTURE.md) — crate/module structure and data flow
- [MIGRATION](docs/MIGRATION.md) — how the four legacy repos were merged
- [PARAMETERS](docs/PARAMETERS.md) — full parameter ID reference
- [BUILDING](docs/BUILDING.md) — toolchain, scripts, CI, bundling
- [MODULES](docs/MODULES.md) — writing native/Lua/compiled modules
- [LUA](docs/LUA.md) — Lua patch format reference
- [SETUPS](docs/SETUPS.md) — synth setup JSON configs and routing

## Testing

```bash
cargo test -p modulus-core                   # DSP + module engine tests
$env:MODULUS_DEMO_MODULE = "$PWD\target\debug\demo_module.dll"  # Windows
cargo build -p demo-module && cargo test -p modulus-core --test plugin_host
cargo run -p modulus-core --example patch_player   # renders a Lua patch -> WAV
```

## CI / Releases

- `.github/workflows/build.yml` — fmt, clippy, tests, bundles (all platforms)
- `.github/workflows/release.yml` — tag-triggered; OpenCode audits/notes the
  release, bundles are uploaded to the GitHub Release
- `.github/workflows/opencode.yml` — `/opencode` or `/oc` in issues/PRs runs
  OpenCode on the GitHub runner

Build jobs are shared composite actions under `.github/actions/`.

## License

ISC. See the repository license files.

<sub>Built with the Modulus module engine — osc/filter/env/fx modules expose
the same Rust trait whether they come from the built-ins, a Lua patch, or a
compiled shared library.</sub>