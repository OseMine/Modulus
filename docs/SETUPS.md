# Modulus — Synth Setup Configs

A **synth setup** is a JSON file that defines how the synth is set up and
routed internally: which sound generators feed the mixer, the filter, the
amp and filter envelopes, and the modulator. Setups live in `setups/`
(per-user: `Modulus/setups` on Windows, `~/.local/share/Modulus/setups`
on Linux, `~/Library/Application Support/Modulus/setups` on macOS).

Compiles into a `SynthGraph`: real `AudioModule` instances wired in a fixed
topology, so rendering stays allocation-free and lock-free.

## The default setup

```
soundgen 1────┐
soundgen 2────┤  mix  │ pregain │ filter │ amp env ──► out
soundgen 3────┼──► ──► ────────► ────────► ────────
soundgen 4────┘   │         │                      ▲
                   │         └─ filter env (contour)│
                   └────────────► modulator ────────┘
                                    │  to_amp
                                    └─► to_filter_octaves
```

The default configuration is 4 oscillators/sound generators (model free of
choice) into the mixer, then a filter (model free of choice) with pregain,
an amp envelope and a filter envelope with contour, and a modulator that
can modulate both amp and filter with configurable amounts.

## File format

```json
{
  "name": "Default 4-Osc",
  "soundgens": [
    { "params": { "waveform": 4, "level": 0.25, "pitch_semitones": 0 } },
    { "params": { "waveform": 0, "level": 0.25, "pitch_semitones": 7 } }
  ],
  "mixer":  { "output_level": 1.0 },
  "filter": {
    "params": { "filter_type": 0, "cutoff": 2000, "resonance": 0.25 },
    "pregain_db": 0.0
  },
  "amp_envelope": {
    "params": { "attack": 0.01, "decay": 0.1, "sustain": 0.6, "release": 0.2 }
  },
  "filter_envelope": {
    "params": { "attack": 0.02, "decay": 0.3, "sustain": 0.4, "release": 0.4 },
    "contour_octaves": 2.0
  },
  "modulator": {
    "params": { "waveform": 0, "rate_hz": 4.0, "depth": 0.7 },
    "to_amp": 0.12,
    "to_filter_octaves": 1.0
  }
}
```

Fields:

- `soundgens` — array of slots. Missing/empty = the default 4-osc bank
  (VA saw +7, sine, saw −5, VA saw +12).
- `mixer.output_level` — master bus level.
- `filter` — `params` (model parameters), `pregain_db` (drive into the
  filter).
- `filter_envelope.contour_octaves` — how many octaves the filter envelope
  sweeps the cutoff (`cutoff * 2^(contour_octaves * env)`).
- `modulator.to_amp` — amp modulation amount (0..1; 1 = full tremolo),
  `modulator.to_filter_octaves` — filter modulation in octaves.

### Model predefinition (optional)

Every slot may set a `model` — a registered module name (e.g.
`am_bridge`, `fm_bridge`, `filter`, `envelope`, `lfo`, or a compiled
module). The role default is used when `model` is empty:

| Role | Default model |
| ---- | ------------- |
| soundgen | `oscillator` |
| filter | `filter` |
| amp / filter envelope | `envelope` |
| modulator | `lfo` |

Slots are validated against their role at build time (e.g. an LFO cannot
fill a soundgen slot). See `setups/bridge_lead.json` for a setup that
predefines an `am_bridge` and a `lfo`.

## Shipped setups

- `setups/default.json` — the 4-soundgen subtracted default.
- `setups/bridge_lead.json` — AM-bridge lead (`am_bridge` + `lfo`).
- `setups/juno_106.json` — Juno-106 style: single VA-saw DCO into a
  Roland LP24 with 2-octave filter envelope, resonance, LFO wobble.
- `setups/dx7.json` — DX7 style: two-operator FM pairs (`fm_bridge`,
  2:1 and ≈7:1 ratios) into an open filter with a percussive decay.

## Rendering

```rust
use modulus_core::modules::builtin_registry;
use modulus_core::synth_setup::SynthSetup;

let setup = SynthSetup::load(std::path::Path::new("setups/default.json"))?;
let mut graph = setup.build(&builtin_registry())?;   // -> SynthGraph
graph.prepare(44_100.0);
graph.note_on(60, 1.0, 440.0);
graph.process_frame(&mut frame, &events, 44_100.0);
```

Or offline:

```bash
cargo run -p modulus-core --example setup_player
# MODULUS_SETUP=setups/bridge_lead.json MODULUS_OUTPUT=target/bridge.wav ...
```

## Modulation

Modulation happens through `AudioModule::cv()`: envelopes expose their
current stage (0..1), modulators a CV centered around `1` (`1 ± depth`, so
modulation vanishes at `depth` 0 and the source is symmetric around the
base value). `SynthGraph` applies:

- filter cutoff: `cutoff * 2^(contour_octaves * env) *
  2^(to_filter_octaves * (mod_cv − 1))` — the LFO moves the cutoff
  `± to_filter_octaves` octaves around the base value.
- amp: multiplied by `clamp(1 + (mod_cv − 1) * to_amp, 0, 1)` after the
  amp envelope (1 = full tremolo).