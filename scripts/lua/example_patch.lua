return {
  name = "Deep Pad",
  modules = {
    { kind = "oscillator", id = "osc1", waveform = 4, level = 0.5, pitch_semitones = 0 },
    { kind = "oscillator", id = "osc2", waveform = 0, level = 0.2, pitch_semitones = 7 },
    { kind = "filter",     id = "filt", filter_type = 0, cutoff = 2000, resonance = 0.3 },
    { kind = "envelope",   id = "env",  attack = 0.05, decay = 0.2, sustain = 0.6, release = 0.4 },
    { kind = "chorus",     id = "ch",   dry_wet = 0.3, depth = 0.4, rate = 1.0, voices = 3, delay_ms = 12, width = 0.5 },
    { kind = "gain",       id = "out",  gain_db = -6 },
  },
}