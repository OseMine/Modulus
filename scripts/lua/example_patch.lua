return {
  name = "Deep Pad",
  modules = {
    { kind = "am_bridge", id = "bridge",
      carrier_waveform = 4, carrier_level = 0.5, carrier_pitch = 0,
      modulator_waveform = 0, modulator_level = 0.5, modulator_pitch = 7,
      mode = 1, am_depth = 0.5 },
    { kind = "lfo",       id = "trem", waveform = 0, rate_hz = 4, depth = 0.2 },
    { kind = "envelope",  id = "env",  attack = 0.05, decay = 0.2, sustain = 0.6, release = 0.4 },
    { kind = "filter",    id = "filt", filter_type = 0, cutoff = 2000, resonance = 0.3 },
    { kind = "chorus",    id = "ch",   dry_wet = 0.3, depth = 0.4, rate = 1.0, voices = 3, delay_ms = 12, width = 0.5 },
    { kind = "gain",      id = "out",  gain_db = -6 },
  },
}