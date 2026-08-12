//! Synth setup presets selectable from the editor.
//!
//! The bundled `setups/*.json` configs are compiled into the plugin and
//! presented in a dropdown. Selecting one maps the setup's slot/params onto
//! the plugin's `ModulusParams` through the `ParamSetter` (same mechanism as
//! the sliders), so a click re-tunes the whole engine.

use modulus_core::synth_setup::{ModuleSlot, SynthSetup};
use nih_plug::prelude::ParamSetter;
use nih_plug::prelude::{EnumParam, FloatParam, IntParam};

use crate::params::{ModulusParams, ParamFilterType, ParamOsc2Mode, ParamWaveform};

/// A shipped setup config: display name + embedded JSON.
pub struct ShippedSetup {
    pub name: &'static str,
    pub json: &'static str,
}

/// Every `setups/*.json` bundled into the plugin. Keep in sync with the
/// repository's `setups/` folder (the parse test below verifies the JSON).
pub const SHIPPED: &[ShippedSetup] = &[
    ShippedSetup {
        name: "Default 4-Osc",
        json: include_str!("../../../setups/default.json"),
    },
    ShippedSetup {
        name: "Bridge Lead",
        json: include_str!("../../../setups/bridge_lead.json"),
    },
    ShippedSetup {
        name: "Juno 106 Style",
        json: include_str!("../../../setups/juno_106.json"),
    },
    ShippedSetup {
        name: "DX7 Style",
        json: include_str!("../../../setups/dx7.json"),
    },
];

/// Parse the setup at `index` (clamped into range).
pub fn parse(index: usize) -> SynthSetup {
    let index = index.min(SHIPPED.len() - 1);
    SynthSetup::from_json(SHIPPED[index].json)
        .expect("shipped setups must parse (tests keep the JSON in sync)")
}

/// Display name of the setup at `index`.
pub fn name(index: usize) -> &'static str {
    SHIPPED[index.min(SHIPPED.len() - 1)].name
}

/// Apply the setup at `index` to the plugin parameters.
pub fn apply(index: usize, params: &ModulusParams, setter: &ParamSetter<'_>) {
    apply_setup(&parse(index), params, setter);
}

/// Map a parsed setup onto the plugin's voice-pool parameters:
///
/// - `soundgens[0]` → OSC 1, `soundgens[1]` → OSC 2. Oscillator slots map
///   `waveform`/`level`/`pitch_semitones`; the AM/FM bridges map their
///   carrier onto OSC 1 and modulator onto OSC 2 (`fm_bridge` also gets the
///   AM mode + its index folded into `osc2_am_depth`).
/// - `filter` params → filter section; `filter_envelope.contour_octaves` →
///   filter env amount (clamped to the plugin range).
/// - amp/filter envelopes map their ADSR directly.
/// - `mixer.output_level` → output gain in dB.
/// - The modulator (LFO) has no plugin counterpart and is skipped.
///
/// Unknown or out-of-range param names are skipped; the plugin's own params
/// remain untouched otherwise.
pub fn apply_setup(setup: &SynthSetup, params: &ModulusParams, setter: &ParamSetter<'_>) {
    let soundgens = setup.effective_soundgens();
    if let Some(slot) = soundgens.first() {
        let osc1 = Osc1Targets {
            waveform: &params.osc1_waveform,
            level: &params.osc1_level,
            pitch: &params.osc1_pitch,
        };
        let osc2 = Osc2Targets {
            waveform: &params.osc2_waveform,
            level: &params.osc2_level,
            pitch: &params.osc2_pitch,
            mode: &params.osc2_mode,
            am_depth: &params.osc2_am_depth,
        };
        apply_soundgen(slot, &osc1, &osc2, setter);
    }

    let filter = &setup.filter.slot.params;
    if let Some(&value) = filter.get("filter_type") {
        setter.set_parameter(
            &params.filt_type,
            ParamFilterType::from_index(value as usize),
        );
    }
    set_float(setter, &params.filt_cutoff, filter.get("cutoff").copied());
    set_float(
        setter,
        &params.filt_resonance,
        filter.get("resonance").copied(),
    );

    let amp = &setup.amp_envelope.params;
    set_float(setter, &params.env_attack, amp.get("attack").copied());
    set_float(setter, &params.env_decay, amp.get("decay").copied());
    set_float(setter, &params.env_sustain, amp.get("sustain").copied());
    set_float(setter, &params.env_release, amp.get("release").copied());

    let fenv = &setup.filter_envelope.slot.params;
    set_float(setter, &params.fenv_attack, fenv.get("attack").copied());
    set_float(setter, &params.fenv_decay, fenv.get("decay").copied());
    set_float(setter, &params.fenv_sustain, fenv.get("sustain").copied());
    set_float(setter, &params.fenv_release, fenv.get("release").copied());

    setter.set_parameter(
        &params.filt_env_amount,
        setup.filter_envelope.contour_octaves.clamp(0.0, 1.0),
    );

    let level_db = if setup.mixer.output_level > 0.0 {
        Some(20.0 * setup.mixer.output_level.log10())
    } else {
        None
    };
    set_float(setter, &params.fx_gain, level_db);
}

/// Param targets for one oscillator slot.
struct Osc1Targets<'a> {
    waveform: &'a EnumParam<ParamWaveform>,
    level: &'a FloatParam,
    pitch: &'a IntParam,
}

/// OSC 2 extends OSC 1 with the bridge mode + AM depth.
struct Osc2Targets<'a> {
    waveform: &'a EnumParam<ParamWaveform>,
    level: &'a FloatParam,
    pitch: &'a IntParam,
    mode: &'a EnumParam<ParamOsc2Mode>,
    am_depth: &'a FloatParam,
}

/// Map one soundgen slot onto OSC 1/2 (+ AM depth). Models without a match
/// are ignored.
fn apply_soundgen(
    slot: &ModuleSlot,
    osc1: &Osc1Targets<'_>,
    osc2: &Osc2Targets<'_>,
    setter: &ParamSetter<'_>,
) {
    let params = &slot.params;
    match slot.model.as_str() {
        // Plain oscillator: OSC 1 only.
        "" | "oscillator" | "oscillator2" => {
            set_waveform(setter, osc1.waveform, params.get("waveform").copied());
            set_float(setter, osc1.level, params.get("level").copied());
            set_pitch(setter, osc1.pitch, params.get("pitch_semitones").copied());
        }
        // Bridge: carrier -> OSC 1, modulator -> OSC 2.
        "am_bridge" | "fm_bridge" => {
            set_waveform(
                setter,
                osc1.waveform,
                params.get("carrier_waveform").copied(),
            );
            set_float(setter, osc1.level, params.get("carrier_level").copied());
            set_pitch(setter, osc1.pitch, params.get("carrier_pitch").copied());
            set_waveform(
                setter,
                osc2.waveform,
                params.get("modulator_waveform").copied(),
            );
            set_float(setter, osc2.level, params.get("modulator_level").copied());
            set_pitch(setter, osc2.pitch, params.get("modulator_pitch").copied());
            setter.set_parameter(osc2.mode, ParamOsc2Mode::Am);
            // FM depth is a phase-modulation index; the plugin exposes an AM
            // depth, so fold it in (clamped) to keep the patch audible.
            set_float(
                setter,
                osc2.am_depth,
                params
                    .get("fm_amount")
                    .or_else(|| params.get("am_depth"))
                    .copied(),
            );
        }
        _ => {}
    }
}

fn set_float(setter: &ParamSetter<'_>, param: &FloatParam, value: Option<f32>) {
    if let Some(value) = value {
        setter.set_parameter(param, value);
    }
}

fn set_pitch(setter: &ParamSetter<'_>, param: &IntParam, value: Option<f32>) {
    if let Some(value) = value {
        setter.set_parameter(param, value.clamp(-24.0, 24.0) as i32);
    }
}

fn set_waveform(setter: &ParamSetter<'_>, param: &EnumParam<ParamWaveform>, value: Option<f32>) {
    if let Some(value) = value {
        setter.set_parameter(
            param,
            ParamWaveform::from_index(value.clamp(0.0, 7.0) as usize),
        );
    }
}

impl ParamWaveform {
    fn from_index(index: usize) -> Self {
        match index {
            1 => ParamWaveform::Saw,
            2 => ParamWaveform::Square,
            3 => ParamWaveform::AnalogSaw,
            4 => ParamWaveform::VASaw,
            5 => ParamWaveform::AnalogSquare,
            6 => ParamWaveform::VASquare,
            7 => ParamWaveform::VintageSaw,
            _ => ParamWaveform::Sine,
        }
    }
}

impl ParamFilterType {
    fn from_index(index: usize) -> Self {
        match index {
            1 => ParamFilterType::Roland,
            2 => ParamFilterType::Le13700,
            3 => ParamFilterType::Arp4075,
            _ => ParamFilterType::Moog,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_setups_parse_and_have_names() {
        assert!(SHIPPED.len() >= 4, "expected at least the 4 shipped setups");
        for (index, shipped) in SHIPPED.iter().enumerate() {
            assert!(!shipped.name.is_empty(), "setup {index} needs a name");
            let setup = SynthSetup::from_json(shipped.json)
                .unwrap_or_else(|err| panic!("setup {} should parse: {err}", shipped.name));
            assert_eq!(setup.name, shipped.name, "name mismatch in setups.rs");
        }
    }
}
