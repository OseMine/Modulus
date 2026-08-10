# Modulus — Parameter Reference

All parameter IDs are stable identifiers used by the host. They use a flat,
prefixed naming scheme. `smoothing` refers to `SmoothingStyle` configured on
the param (Hz for logarithmic smoothing).

## Modulus (synthesizer) — 30 parameters

### Global

| ID | Type | Range | Default | Unit |
| --- | ---- | ----- | ------- | ---- |
| `global_tuning` | float | 415–465 Hz | 440 | Hz |

### Oscillator 1

| ID | Type | Range | Default | Notes |
| -- | ---- | ----- | ------- | ----- |
| `osc1_waveform` | enum | Sine, Saw, Square, Analog Saw, VA Saw, Analog Square, VA Square, Vintage Saw | Sine | |
| `osc1_level` | float | 0–1 | 0.7 | smoothed |
| `osc1_pitch` | int | −24…24 | 0 | semitones |

### Oscillator 2

| ID | Type | Range | Default | Notes |
| -- | ---- | ----- | ------- | ----- |
| `osc2_waveform` | enum | same as osc1 | Saw | |
| `osc2_level` | float | 0–1 | 0.5 | smoothed |
| `osc2_pitch` | int | −24…24 | 0 | semitones |
| `osc2_mode` | enum | Mix / AM | Mix | AM bridges osc2 into osc1 |
| `osc2_am_depth` | float | 0–1 | 0.5 | smoothed |

### Filter

| ID | Type | Range | Default | Notes |
| -- | ---- | ----- | ------- | ----- |
| `filt_type` | enum | Moog, Roland, LE13700, ARP 4075 | Moog | |
| `filt_cutoff` | float | 20–20 kHz skewed | 1 kHz | logarithmic smoother 30 ms |
| `filt_resonance` | float | 0–1 | 0.3 | |
| `filt_env_amount` | float | 0–1 | 0.0 | |

### Amplitude envelope

| ID | Type | Range | Default | Unit |
| -- | ---- | ----- | ------- | ---- |
| `env_attack` | float | 1 ms–1 s skewed | 10 ms | s |
| `env_decay` | float | 1 ms–1 s skewed | 100 ms | s |
| `env_sustain` | float | 0–1 | 0.5 | |
| `env_release` | float | 1 ms–1 s skewed | 100 ms | s |

### Filter envelope

| ID | Type | Range | Default | Unit |
| -- | ---- | ----- | ------- | ---- |
| `fenv_attack` | float | 1 ms–1 s skewed | 10 ms | s |
| `fenv_decay` | float | 1 ms–1 s skewed | 100 ms | s |
| `fenv_sustain` | float | 0–1 | 0.5 | |
| `fenv_release` | float | 1 ms–1 s skewed | 100 ms | s |

### Chorus / output

| ID | Type | Range | Default | Notes |
| -- | ---- | ----- | ------- | ----- |
| `fx_enable` | bool | | true | |
| `fx_chorus_dry_wet` | float | 0–1 | 0.35 | smoothed |
| `fx_chorus_depth` | float | 0–1 | 0.5 | smoothed |
| `fx_chorus_rate` | float | 0.1–10 Hz | 1.0 | smoothed, unit Hz |
| `fx_chorus_voices` | int | 0–5 | 2 | (0 = chorus bypassed) |
| `fx_chorus_delay` | float | 0–50 ms | 10 | unit ms |
| `fx_chorus_width` | float | 0–1 | 0.5 | smoothed |
| `fx_gain` | float | −60…12 dB | 0 | logarithmic smoother |

## Modulus FX (effect) — 15 parameters

### Filter

| ID | Type | Range | Default | Notes |
| -- | ---- | ----- | ------- | ----- |
| `filt_type` | enum | Moog, Roland, LE13700, ARP 4075 | Moog | |
| `filt_cutoff` | float | 20 Hz–20 kHz | 1 kHz | unit Hz, dB-scaled display |
| `filt_resonance` | float | 0–1 | 0.0 | |
| `filt_smoothing` | float | 0–1000 ms | 50 | smoothing time constant |
| `filt_enabled` | bool | | true | |

### Chorus

| ID | Type | Range | Default | Notes |
| -- | ---- | ----- | ------- | ----- |
| `chorus_enabled` | bool | | true | |
| `chorus_dry_wet` | float | 0–1 | 0.5 | smoothed |
| `chorus_depth` | float | 0–1 | 0.5 | smoothed |
| `chorus_rate` | float | 0.1–10 Hz | 1.0 | smoothed, unit Hz |
| `chorus_voices` | int | 0–5 | 2 | (0 = chorus bypassed) |
| `chorus_delay` | float | 0–50 ms | 10 | unit ms |
| `chorus_width` | float | 0–1 | 0.5 | smoothed |

### Gain

| ID | Type | Range | Default | Notes |
| -- | ---- | ----- | ------- | ----- |
| `gain_in` | float | −60…12 dB | 0 | logarithmic smoother |
| `gain_out` | float | −60…12 dB | 0 | logarithmic smoother |

## Enum parameter values

Waveforms (both `*_waveform` params, in index order):

1. `Sine`
2. `Saw`
3. `Square`
4. `Analog Saw`
5. `VA Saw`
6. `Analog Square`
7. `VA Square`
8. `Vintage Saw`

Filter types (both plugins, `filt_type`): `Moog`, `Roland`, `LE13700`,
`ARP 4075`.

`osc2_mode`: `Mix`, `AM`.

Matching core enums live in `modulus-core`: `waveform::Waveform` (8),
`filter::FilterType` (4), `voice::Osc2Mode` (2). They index in the same
order as the plugin enums above.