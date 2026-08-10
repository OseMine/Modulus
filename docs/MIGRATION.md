# Modulus — Migration Notes

How the four legacy repositories were merged into this workspace, and what
changed along the way.

## Source repositories (archived under `workspace/`, not built)

| Repo                 | Contribution                          | Notes |
| -------------------- | ------------------------------------- | ----- |
| `variable-synth`     | phase-accumulator oscillators, voice pool, envelopes, filter wiring | main synth architecture |
| `Am-Synth`           | AM bridge between oscillators, 8-voice management | AM mode, corrected ADSR release |
| `variable-filter`    | 4-model ladder filter (Moog/Roland/LE13700/ARP4075) | unified into one coefficient set |
| `variable-effects`   | chorus + gain chain                   | placeholder chorus replaced |

All four pinned the **same nih-plug git rev**
(`dfafe90349aa3d8e40922ec031b6d673803d6432`), which is what made a clean
workspace merge possible.

## Module-by-module mapping

| Legacy file                     | `modulus-core` destination   | Changes |
| ------------------------------- | ---------------------------- | ------- |
| `variable-synth` `oscillator.rs` | `oscillator.rs`, `waveform.rs` | 8 waveform generators consolidated; `FastRng` replaces `rand::thread_rng()` (RT-safe) |
| `variable-synth` `voice.rs`      | `voice.rs`                   | `VoicePool::process` returns summed mono; stealing round-robin |
| `Am-Synth` `voice.rs`            | `voice.rs` (AM bridge)       | `Osc2Mode::Am` + `osc2_am_depth` multiplied into osc2 output |
| `Am-Synth` `envelope.rs`         | `envelope.rs`                | release now decays from the current value, not from sustain |
| `variable-filter` `filter.rs`    | `filter.rs`                  | 4 models share one ladder core; `OnePoleSmoother` replaces `static mut` |
| `variable-effects` `fx.rs`       | `fx.rs`                      | naive chorus replaced by multi-tap modulated delay lines; `Box<dyn Effect>` removed |
| any of the above `params.rs`     | plugin `params.rs` files     | flat prefixed IDs (see PARAMETERS.md) |

## Structural decisions

1. **Single shared crate.** Everything DSP lives in `modulus-core`; the two
   plugins are thin nih-plug adapters (`params` + `process` + `editor`).
   Duplicated DSP units (two oscillator implementations, two envelope
   implementations) were merged into one each.
2. **Zero-allocation audio path.** The old repos allocated per-note (Vec in
   `variable-effects`) or used thread-local RNG. All of that was removed;
   see the real-time rules in ARCHITECTURE.md.
3. **Sample-accurate MIDI.** `Am-Synth` handled MIDI after the audio block;
   Modulus applies events in-sample via `context.next_event()`.
4. **`velocity` semantics.** At the pinned nih-plug rev, `NoteOn.velocity`
   is already normalized `f32` (0..1); the old `/ 127.0` divisors were
   removed.
5. **`iter_samples()` API.** At this rev the iterator yields
   `ChannelSamples` (channels with `iter_mut()`); the code was written
   against that shape.
6. **VST3 class ids** must be exactly 16 bytes: synth uses
   `*b"ModulusSynth...."`, FX `*b"ModulusFXPlugin."`.

## Things deliberately not carried over

- The `Am-Synth` carrier/modulator/global filter *banks* were never wired
  into its audio path; Modulus uses one per-voice filter + one shared
  effects section.
- Preset files / DAW project references from the old repos.
- The old `rand` dependency and any per-sample allocations.

## Build-system changes

- `anymap 1.0.0-beta.2` (dependency of the pinned nih-plug) fails to compile
  on current rustc (E0804). A fixed copy is vendored at `vendor/anymap` and
  wired in via `[patch.crates-io]` in the root `Cargo.toml`.
- `xtask` performs the bundle step (`cargo run -p xtask --release bundle`)
  and is cross-platform (Windows `x86_64-win`, macOS `macOS`, Linux
  `x86_64-linux`).