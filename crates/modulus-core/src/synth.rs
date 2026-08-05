use crate::fx::{apply_gain_db, Chorus, SynthFxParams};
use crate::voice::{SynthFrameParams, VoicePool};

/// The complete Modulus synth engine: polyphonic voice pool followed by
/// the shared effects section (chorus + output gain).
pub struct SynthEngine {
    pool: VoicePool,
    chorus: Chorus,
}

impl SynthEngine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            pool: VoicePool::new(sample_rate),
            chorus: Chorus::new(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.pool.set_sample_rate(sample_rate);
        self.chorus.set_sample_rate(sample_rate);
    }

    pub fn reset(&mut self) {
        self.pool.reset();
        self.chorus.reset();
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.pool.note_on(note, velocity);
    }

    pub fn note_off(&mut self, note: u8) {
        self.pool.note_off(note);
    }

    pub fn process(
        &mut self,
        params: &SynthFrameParams,
        fx_params: &SynthFxParams,
        sample_rate: f32,
    ) -> [f32; 2] {
        let mono = self.pool.process(params);
        let mut frame = [mono, mono];

        if fx_params.chorus_enabled {
            self.chorus.process(&mut frame, &fx_params.chorus, sample_rate);
        }

        apply_gain_db(&mut frame, fx_params.gain_db);
        frame
    }
}
