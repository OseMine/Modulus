use modulus_core::filter::FilterType as CoreFilterType;
use modulus_core::voice::Osc2Mode as CoreOsc2Mode;
use modulus_core::waveform::Waveform as CoreWaveform;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

#[derive(Enum, PartialEq, Clone, Copy, Debug)]
pub enum ParamWaveform {
    #[name = "Sine"]
    Sine,
    #[name = "Saw"]
    Saw,
    #[name = "Square"]
    Square,
    #[name = "Analog Saw"]
    AnalogSaw,
    #[name = "VA Saw"]
    VASaw,
    #[name = "Analog Square"]
    AnalogSquare,
    #[name = "VA Square"]
    VASquare,
    #[name = "Vintage Saw"]
    VintageSaw,
}

impl ParamWaveform {
    pub fn to_core(self) -> CoreWaveform {
        match self {
            ParamWaveform::Sine => CoreWaveform::Sine,
            ParamWaveform::Saw => CoreWaveform::Saw,
            ParamWaveform::Square => CoreWaveform::Square,
            ParamWaveform::AnalogSaw => CoreWaveform::AnalogSaw,
            ParamWaveform::VASaw => CoreWaveform::VASaw,
            ParamWaveform::AnalogSquare => CoreWaveform::AnalogSquare,
            ParamWaveform::VASquare => CoreWaveform::VASquare,
            ParamWaveform::VintageSaw => CoreWaveform::VintageSaw,
        }
    }
}

#[derive(Enum, PartialEq, Clone, Copy, Debug)]
pub enum ParamOsc2Mode {
    #[name = "Mix"]
    Mix,
    #[name = "AM"]
    Am,
}

impl ParamOsc2Mode {
    pub fn to_core(self) -> CoreOsc2Mode {
        match self {
            ParamOsc2Mode::Mix => CoreOsc2Mode::Mix,
            ParamOsc2Mode::Am => CoreOsc2Mode::Am,
        }
    }
}

#[derive(Enum, PartialEq, Clone, Copy, Debug)]
pub enum ParamFilterType {
    #[name = "Moog"]
    Moog,
    #[name = "Roland"]
    Roland,
    #[name = "LE13700"]
    Le13700,
    #[name = "ARP 4075"]
    Arp4075,
}

impl ParamFilterType {
    pub fn to_core(self) -> CoreFilterType {
        match self {
            ParamFilterType::Moog => CoreFilterType::Moog,
            ParamFilterType::Roland => CoreFilterType::Roland,
            ParamFilterType::Le13700 => CoreFilterType::Le13700,
            ParamFilterType::Arp4075 => CoreFilterType::Arp4075,
        }
    }
}

#[derive(Params)]
pub struct ModulusParams {
    /// The editor state, saved together with the parameter state so the window
    /// size can be restored.
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "global_tuning"]
    pub global_tuning: FloatParam,

    #[id = "osc1_waveform"]
    pub osc1_waveform: EnumParam<ParamWaveform>,
    #[id = "osc1_level"]
    pub osc1_level: FloatParam,
    #[id = "osc1_pitch"]
    pub osc1_pitch: IntParam,

    #[id = "osc2_waveform"]
    pub osc2_waveform: EnumParam<ParamWaveform>,
    #[id = "osc2_level"]
    pub osc2_level: FloatParam,
    #[id = "osc2_pitch"]
    pub osc2_pitch: IntParam,
    #[id = "osc2_mode"]
    pub osc2_mode: EnumParam<ParamOsc2Mode>,
    #[id = "osc2_am_depth"]
    pub osc2_am_depth: FloatParam,

    #[id = "filt_type"]
    pub filt_type: EnumParam<ParamFilterType>,
    #[id = "filt_cutoff"]
    pub filt_cutoff: FloatParam,
    #[id = "filt_resonance"]
    pub filt_resonance: FloatParam,
    #[id = "filt_env_amount"]
    pub filt_env_amount: FloatParam,

    #[id = "env_attack"]
    pub env_attack: FloatParam,
    #[id = "env_decay"]
    pub env_decay: FloatParam,
    #[id = "env_sustain"]
    pub env_sustain: FloatParam,
    #[id = "env_release"]
    pub env_release: FloatParam,

    #[id = "fenv_attack"]
    pub fenv_attack: FloatParam,
    #[id = "fenv_decay"]
    pub fenv_decay: FloatParam,
    #[id = "fenv_sustain"]
    pub fenv_sustain: FloatParam,
    #[id = "fenv_release"]
    pub fenv_release: FloatParam,

    #[id = "fx_enable"]
    pub fx_enable: BoolParam,
    #[id = "fx_chorus_dry_wet"]
    pub fx_chorus_dry_wet: FloatParam,
    #[id = "fx_chorus_depth"]
    pub fx_chorus_depth: FloatParam,
    #[id = "fx_chorus_rate"]
    pub fx_chorus_rate: FloatParam,
    #[id = "fx_chorus_voices"]
    pub fx_chorus_voices: IntParam,
    #[id = "fx_chorus_delay"]
    pub fx_chorus_delay: FloatParam,
    #[id = "fx_chorus_width"]
    pub fx_chorus_width: FloatParam,
    #[id = "fx_gain"]
    pub fx_gain: FloatParam,
}

impl Default for ModulusParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(640, 520),

            global_tuning: tuning_param("Tuning", 440.0),

            osc1_waveform: EnumParam::new("OSC 1 Waveform", ParamWaveform::Sine),
            osc1_level: level_param("OSC 1 Level", 0.7),
            osc1_pitch: IntParam::new("OSC 1 Pitch", 0, IntRange::Linear { min: -24, max: 24 })
                .with_unit(" st"),

            osc2_waveform: EnumParam::new("OSC 2 Waveform", ParamWaveform::Saw),
            osc2_level: level_param("OSC 2 Level", 0.5),
            osc2_pitch: IntParam::new("OSC 2 Pitch", 0, IntRange::Linear { min: -24, max: 24 })
                .with_unit(" st"),
            osc2_mode: EnumParam::new("OSC 2 Mode", ParamOsc2Mode::Mix),
            osc2_am_depth: level_param("OSC 2 AM Depth", 0.5),

            filt_type: EnumParam::new("Filter Type", ParamFilterType::Moog),
            filt_cutoff: cutoff_param("Filter Cutoff", 1000.0)
                .with_smoother(SmoothingStyle::Logarithmic(30.0)),
            filt_resonance: level_param("Filter Resonance", 0.3),
            filt_env_amount: level_param("Filter Env Amount", 0.0),

            env_attack: time_param("Env Attack", 0.01),
            env_decay: time_param("Env Decay", 0.1),
            env_sustain: level_param("Env Sustain", 0.5),
            env_release: time_param("Env Release", 0.1),

            fenv_attack: time_param("Filter Env Attack", 0.01),
            fenv_decay: time_param("Filter Env Decay", 0.1),
            fenv_sustain: level_param("Filter Env Sustain", 0.5),
            fenv_release: time_param("Filter Env Release", 0.1),

            fx_enable: BoolParam::new("FX Enable", true),
            fx_chorus_dry_wet: level_param("Chorus Dry/Wet", 0.35),
            fx_chorus_depth: level_param("Chorus Depth", 0.5),
            fx_chorus_rate: FloatParam::new(
                "Chorus Rate",
                1.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" Hz"),
            fx_chorus_voices: IntParam::new(
                "Chorus Voices",
                2,
                IntRange::Linear { min: 0, max: 5 },
            ),
            fx_chorus_delay: FloatParam::new(
                "Chorus Delay",
                10.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 50.0,
                },
            )
            .with_unit(" ms"),
            fx_chorus_width: level_param("Chorus Width", 0.5),
            fx_gain: gain_param("Output Gain", 0.0),
        }
    }
}

fn tuning_param(name: &'static str, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Linear {
            min: 415.0,
            max: 465.0,
        },
    )
    .with_unit(" Hz")
    .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
    .with_string_to_value(formatters::s2v_f32_hz_then_khz())
}

fn cutoff_param(name: &'static str, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Skewed {
            min: 20.0,
            max: 20_000.0,
            factor: FloatRange::skew_factor(-2.0),
        },
    )
    .with_unit(" Hz")
    .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
    .with_string_to_value(formatters::s2v_f32_hz_then_khz())
}

fn level_param(name: &'static str, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max: 1.0 })
        .with_smoother(SmoothingStyle::Linear(50.0))
}

fn time_param(name: &'static str, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Skewed {
            min: 0.001,
            max: 1.0,
            factor: 0.5,
        },
    )
    .with_unit(" s")
}

fn gain_param(name: &'static str, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Linear {
            min: -60.0,
            max: 12.0,
        },
    )
    .with_smoother(SmoothingStyle::Logarithmic(50.0))
    .with_unit(" dB")
}
