use modulus_core::filter::FilterType as CoreFilterType;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

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
pub struct ModulusFxParams {
    /// The editor state, saved together with the parameter state so the window
    /// size can be restored.
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "filt_type"]
    pub filt_type: EnumParam<ParamFilterType>,
    #[id = "filt_cutoff"]
    pub filt_cutoff: FloatParam,
    #[id = "filt_resonance"]
    pub filt_resonance: FloatParam,
    #[id = "filt_smoothing"]
    pub filt_smoothing: FloatParam,
    #[id = "filt_enabled"]
    pub filt_enabled: BoolParam,

    #[id = "chorus_enabled"]
    pub chorus_enabled: BoolParam,
    #[id = "chorus_dry_wet"]
    pub chorus_dry_wet: FloatParam,
    #[id = "chorus_depth"]
    pub chorus_depth: FloatParam,
    #[id = "chorus_rate"]
    pub chorus_rate: FloatParam,
    #[id = "chorus_voices"]
    pub chorus_voices: IntParam,
    #[id = "chorus_delay"]
    pub chorus_delay: FloatParam,
    #[id = "chorus_width"]
    pub chorus_width: FloatParam,

    #[id = "gain_in"]
    pub gain_in: FloatParam,
    #[id = "gain_out"]
    pub gain_out: FloatParam,
}

impl Default for ModulusFxParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(520, 480),

            filt_type: EnumParam::new("Filter Type", ParamFilterType::Moog),
            filt_cutoff: FloatParam::new(
                "Filter Cutoff",
                1000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz())
            .with_smoother(SmoothingStyle::Logarithmic(30.0)),
            filt_resonance: FloatParam::new(
                "Filter Resonance",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),
            filt_smoothing: FloatParam::new(
                "Filter Smoothing",
                50.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1000.0,
                },
            )
            .with_unit(" ms"),
            filt_enabled: BoolParam::new("Filter Enabled", true),

            chorus_enabled: BoolParam::new("Chorus Enabled", true),
            chorus_dry_wet: FloatParam::new(
                "Chorus Dry/Wet",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),
            chorus_depth: FloatParam::new(
                "Chorus Depth",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),
            chorus_rate: FloatParam::new(
                "Chorus Rate",
                1.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" Hz"),
            chorus_voices: IntParam::new("Chorus Voices", 2, IntRange::Linear { min: 1, max: 8 }),
            chorus_delay: FloatParam::new(
                "Chorus Delay",
                10.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 50.0,
                },
            )
            .with_unit(" ms"),
            chorus_width: FloatParam::new(
                "Chorus Width",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            gain_in: gain_param("Gain In", 0.0),
            gain_out: gain_param("Gain Out", 0.0),
        }
    }
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
