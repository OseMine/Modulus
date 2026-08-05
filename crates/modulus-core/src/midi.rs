/// Convert a MIDI note number to a frequency in Hz, referenced to a tuning.
pub fn midi_note_to_freq(note: u8, tuning_hz: f32) -> f32 {
    tuning_hz * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}
