use crate::envelope::Adsr;
use crate::filter::{VariableFilter, FILTER_MAX_CUTOFF, FILTER_MIN_CUTOFF};
use crate::midi::midi_note_to_freq;
use crate::oscillator::Oscillator;
use crate::waveform::Waveform;

pub const MAX_VOICES: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Osc2Mode {
    Mix,
    Am,
}

impl Osc2Mode {
    pub const ALL: [Osc2Mode; 2] = [Osc2Mode::Mix, Osc2Mode::Am];

    pub const fn name(self) -> &'static str {
        match self {
            Osc2Mode::Mix => "Mix",
            Osc2Mode::Am => "AM",
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Osc2Mode::Mix,
            _ => Osc2Mode::Am,
        }
    }
}

/// Per-sample snapshot of every synthesizer parameter.
///
/// Built once per sample by the plugin from its smoothed `nih-plug`
/// parameters, this keeps the DSP layer framework-agnostic.
#[derive(Clone, Copy, Debug)]
pub struct SynthFrameParams {
    pub osc1_waveform: Waveform,
    pub osc1_level: f32,
    pub osc1_pitch_semitones: i32,
    pub osc2_waveform: Waveform,
    pub osc2_level: f32,
    pub osc2_pitch_semitones: i32,
    pub osc2_mode: Osc2Mode,
    pub osc2_am_depth: f32,
    pub filter_type: crate::filter::FilterType,
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub filter_env_amount: f32,
    pub amp_attack: f32,
    pub amp_decay: f32,
    pub amp_sustain: f32,
    pub amp_release: f32,
    pub filt_attack: f32,
    pub filt_decay: f32,
    pub filt_sustain: f32,
    pub filt_release: f32,
    pub tuning_hz: f32,
}

/// One polyphonic voice: dual oscillators, AM bridge, ladder filter and
/// dual ADSR envelopes. Merges the `variable-synth` waveform engine with
/// the `Am-Synth` dual-oscillator voice architecture.
pub struct Voice {
    pub active: bool,
    pub note: u8,
    pub velocity: f32,
    sample_rate: f32,
    osc1: Oscillator,
    osc2: Oscillator,
    filter: VariableFilter,
    amp_env: Adsr,
    filt_env: Adsr,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            active: false,
            note: 0,
            velocity: 0.0,
            sample_rate,
            osc1: Oscillator::new(sample_rate, Waveform::Sine),
            osc2: Oscillator::new(sample_rate, Waveform::Saw),
            filter: VariableFilter::new(),
            amp_env: Adsr::new(sample_rate),
            filt_env: Adsr::new(sample_rate),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.osc1.set_sample_rate(sample_rate);
        self.osc2.set_sample_rate(sample_rate);
        self.amp_env.set_sample_rate(sample_rate);
        self.filt_env.set_sample_rate(sample_rate);
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.note = 0;
        self.velocity = 0.0;
        self.osc1.reset();
        self.osc2.reset();
        self.filter.reset();
        self.amp_env = Adsr::new(self.sample_rate);
        self.filt_env = Adsr::new(self.sample_rate);
    }

    pub fn trigger(&mut self, note: u8, velocity: f32) {
        self.active = true;
        self.note = note;
        self.velocity = velocity;
        self.osc1.reset();
        self.osc2.reset();
        self.amp_env.trigger();
        self.filt_env.trigger();
    }

    pub fn release(&mut self) {
        self.amp_env.release();
        self.filt_env.release();
    }

    pub fn is_finished(&self) -> bool {
        self.amp_env.is_idle() && self.filt_env.is_idle()
    }

    pub fn process_frame(&mut self, params: &SynthFrameParams) -> f32 {
        let base_frequency = midi_note_to_freq(self.note, params.tuning_hz);
        self.osc1
            .set_frequency(base_frequency * pitch_multiplier(params.osc1_pitch_semitones));
        self.osc2
            .set_frequency(base_frequency * pitch_multiplier(params.osc2_pitch_semitones));
        self.osc1.set_waveform(params.osc1_waveform);
        self.osc2.set_waveform(params.osc2_waveform);

        let sample1 = self.osc1.generate();
        let sample2 = self.osc2.generate();

        let mixed = match params.osc2_mode {
            Osc2Mode::Mix => sample1 * params.osc1_level + sample2 * params.osc2_level,
            Osc2Mode::Am => sample1 * (1.0 + sample2 * params.osc2_am_depth) * params.osc1_level,
        };

        let amp_env = self.amp_env.process();
        let filt_env = self.filt_env.process();

        let cutoff = (params.filter_cutoff
            * 2.0_f32.powf(params.filter_env_amount * filt_env))
            .clamp(FILTER_MIN_CUTOFF, FILTER_MAX_CUTOFF);
        self.filter.set_type(params.filter_type);
        self.filter.set_smoothing(0.0);
        self.filter.set_params(cutoff, params.filter_resonance);
        let filtered = self.filter.process(mixed, self.sample_rate);

        filtered * amp_env * self.velocity
    }
}

/// Fixed-capacity voice pool with round-robin voice stealing, derived from
/// the `Am-Synth` voice management.
pub struct VoicePool {
    voices: [Voice; MAX_VOICES],
    next_steal: usize,
}

impl VoicePool {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice::new(sample_rate)),
            next_steal: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        for voice in &mut self.voices {
            voice.set_sample_rate(sample_rate);
        }
    }

    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
        self.next_steal = 0;
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        if let Some(voice) = self.voices.iter_mut().find(|v| v.active && v.note == note) {
            voice.trigger(note, velocity);
            return;
        }
        if let Some(voice) = self.voices.iter_mut().find(|v| !v.active) {
            voice.trigger(note, velocity);
            return;
        }
        let index = self.next_steal;
        self.next_steal = (self.next_steal + 1) % MAX_VOICES;
        self.voices[index].trigger(note, velocity);
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.active && voice.note == note {
                voice.release();
            }
        }
    }

    pub fn process(&mut self, params: &SynthFrameParams) -> f32 {
        let mut output = 0.0;
        for voice in &mut self.voices {
            if !voice.active {
                continue;
            }
            output += voice.process_frame(params);
            if voice.is_finished() {
                voice.active = false;
            }
        }
        output
    }
}

fn pitch_multiplier(semitones: i32) -> f32 {
    2.0_f32.powf(semitones as f32 / 12.0)
}
