# Modulus — Building

## Toolchain

- Rust stable ≥ 1.97 (rustc + cargo). Verify: `rustc --version`
  - The pinned nih-plug revision requires a recent compiler; older toolchains
    fail on `anymap 1.0.0-beta.2` (E0804) — the vendored patch in
    `vendor/anymap` is already wired through `[patch.crates-io]`.
- C/C++ build tools for the current host (MSVC Build Tools on Windows,
  Xcode CLT on macOS, gcc/clang + pkg-config on Linux) — needed by
  `mlua` (vendored Lua 5.4) and nih-plug's build deps.
- git.

## Build the plugins

```bash
cargo build --release -p modulus-synth -p modulus-fx
```

This produces `target/release/modulus_synth.dll|dylib|so` and
`modulus_fx.dll|dylib|so` (extension depends on host OS).

## Bundle VST3/CLAP

```bash
cargo run -p xtask --release bundle
```

Creates plugin bundles under `target/bundled/`:

```
target/bundled/
  Modulus.vst3/Contents/<os-dir>/Modulus.vst3
  Modulus.clap/Modulus.clap
  Modulus FX.vst3/Contents/<os-dir>/Modulus FX.vst3
  Modulus FX.clap/Modulus FX.clap
```

`<os-dir>` is `x86_64-win` (Windows), `macOS` (macOS), or `x86_64-linux`
(Linux). An optional filter argument builds only one plugin:
`cargo run -p xtask --release bundle fx`.

Point your DAW/plugin host at `target/bundled` to load them.

## One-shot scripts

| Platform | Command | Behavior |
| -------- | ------- | -------- |
| Windows (PowerShell 7+) | `.\scripts\build.ps1` | builds both plugins in release, runs clippy + tests, bundles |
| macOS / Linux | `./scripts/build.sh` | same |

Both scripts accept `--skip-checks` to only build+bundle.

## Tests and lint

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

The module engine tests require the demo module:

```bash
cargo build -p demo-module
# Windows PowerShell:
$env:MODULUS_DEMO_MODULE = "$PWD\target\debug\demo_module.dll"
# macOS/Linux:
export MODULUS_DEMO_MODULE="$PWD/target/debug/libdemo_module.so|dylib"
cargo test -p modulus-core --test plugin_host
```

## Examples

```bash
# Render the example Lua patch to target/patch_output.wav
cargo run -p modulus-core --example patch_player
# Custom patch / output
MODULUS_PATCH=scripts/lua/example_patch.lua MODULUS_OUTPUT=out.wav \
  cargo run -p modulus-core --example patch_player
```

## CI

`.github/workflows/build.yml` runs on every push/PR (Linux, Windows, macOS):
fmt check, clippy `-D warnings`, full test suite, release bundle, artifact
upload. On Windows it additionally builds a `Modulus-Installer-<version>.exe`
(Inno Setup) that installs the VST3/CLAP bundles into the shared
`C:\Program Files\Common Files\{VST3,CLAP}` directories, uploaded as the
`modulus-installer-windows` artifact. `.github/workflows/release.yml` runs on
version tags and attaches bundles to the GitHub Release;
`.github/workflows/opencode.yml` lets OpenCode act on `/oc` / `/opencode`
comments. Shared build logic lives in `.github/actions/`.

## Troubleshooting

| Symptom | Fix |
| ------- | --- |
| `anymap` fails to compile | entries under `vendor/anymap` missing → `git submodule` not applicable; the folder is plain vendored source, `[patch.crates-io]` in root `Cargo.toml` must point at it |
| Linker errors on Windows | install “Desktop development with C++” in Visual Studio Build Tools |
| `mlua` fail to build | ensure a C compiler is available (cc crate) |
| `assert_process_allocs` panic | allocations inside `process()` — see the real-time rules in `docs/ARCHITECTURE.md` |
| `.vst3` doesn't load in DAW | make sure you installed bundles (`target/bundled`), not raw DLLs; on macOS code-sign/notarize locally or use `cargo run -p xtask --release bundle` which sets 755 perms |