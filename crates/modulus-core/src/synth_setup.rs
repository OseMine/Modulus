//! Synth setup configs (feature `setup`).
//!
//! A setup config is a JSON file that defines how the synth is set up and
//! routed internally: which sound generators feed the mixer, the filter
//! model (with pregain), the amp and filter envelopes (with filter
//! contour), and a modulator with its modulation amounts for amp and
//! filter. Every slot may optionally predefine a specific module model;
//! when the `model` field is empty the role's default model is used.
//!
//! ```json
//! {
//!   "name": "Default 4-Osc",
//!   "soundgens": [ { "params": { "waveform": 4, "level": 0.25, "pitch_semitones": 0 } },
//!                  { "params": { "waveform": 0, "level": 0.25, "pitch_semitones": 7 } } ],
//!   "mixer":       { "output_level": 1.0 },
//!   "filter":      { "params": { "filter_type": 0, "cutoff": 2000, "resonance": 0.25 },
//!                    "pregain_db": 0.0 },
//!   "amp_envelope": { "params": { "attack": 0.01, "decay": 0.1, "sustain": 0.6,
//!                                 "release": 0.2 } },
//!   "filter_envelope": { "params": { "attack": 0.02, "decay": 0.3, "sustain": 0.4,
//!                                    "release": 0.4 }, "contour_octaves": 2.0 },
//!   "modulator":  { "params": { "waveform": 0, "rate_hz": 4.0, "depth": 0.7 },
//!                   "to_amp": 0.12, "to_filter_octaves": 1.0 }
//! }
//! ```
//!
//! A setup compiles to a [`SynthGraph`]: real [`AudioModule`] instances
//! wired in a fixed topology, so the render path keeps the usual
//! real-time guarantees (allocation-free, lock-free).
//!
//! Routing:
//!
//! 1. every sound generator adds its output into the shared bus (mixer),
//! 2. the pregain (drive, dB) scales the bus before the filter,
//! 3. the filter processes the bus with `cutoff * 2^(contour * filter_env
//!    + to_filter_octaves * (mod_cv - 1))`,
//! 4. the amp envelope shapes the bus, the modulator scales it by
//!    `1 + (mod_cv - 1) * to_amp`,
//! 5. the mixer output level is applied last.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::fx::gain_db_to_linear;
use crate::modules::registry::ModuleRegistry;
use crate::modules::{AudioModule, ModuleError, ModuleEvents, ModuleKind};

/// Default module model names per role (used when `model` is empty).
pub const DEFAULT_SOUNDGEN_MODEL: &str = "oscillator";
pub const DEFAULT_FILTER_MODEL: &str = "filter";
pub const DEFAULT_ENVELOPE_MODEL: &str = "envelope";
pub const DEFAULT_MODULATOR_MODEL: &str = "lfo";

/// Number of sound generators in the default setup.
pub const DEFAULT_SOUNDGEN_COUNT: usize = 4;

fn one() -> f32 {
    1.0
}
fn default_contour_octaves() -> f32 {
    2.0
}

/// A module instance slot: the model (registry name) to use and the
/// parameters to set on it. An empty `model` selects the role default.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ModuleSlot {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub params: BTreeMap<String, f32>,
}

impl ModuleSlot {
    pub fn effective_model(&self, default: &str) -> String {
        if self.model.is_empty() {
            default.to_string()
        } else {
            self.model.clone()
        }
    }
}

/// The summing bus every sound generator feeds into.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MixerConfig {
    #[serde(default = "one")]
    pub output_level: f32,
}

impl Default for MixerConfig {
    fn default() -> Self {
        Self { output_level: 1.0 }
    }
}

/// The filter stage: model + params, plus the pregain in dB.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FilterConfig {
    #[serde(flatten)]
    pub slot: ModuleSlot,
    /// Drive into the filter, in dB (0 = no pregain).
    #[serde(default)]
    pub pregain_db: f32,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            slot: ModuleSlot::default(),
            pregain_db: 0.0,
        }
    }
}

/// The filter envelope: model + params, plus the filter contour.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FilterEnvelopeConfig {
    #[serde(flatten)]
    pub slot: ModuleSlot,
    /// Filter contour: how many octaves the envelope sweeps the cutoff.
    #[serde(default = "default_contour_octaves")]
    pub contour_octaves: f32,
}

impl Default for FilterEnvelopeConfig {
    fn default() -> Self {
        Self {
            slot: ModuleSlot::default(),
            contour_octaves: 2.0,
        }
    }
}

/// The modulator: model + params, plus its modulation amounts.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ModulatorConfig {
    #[serde(flatten)]
    pub slot: ModuleSlot,
    /// Amount of amplitude modulation (0..1; 1 = full tremolo).
    #[serde(default)]
    pub to_amp: f32,
    /// Amount of filter modulation, in octaves around the base cutoff.
    #[serde(default)]
    pub to_filter_octaves: f32,
}

impl Default for ModulatorConfig {
    fn default() -> Self {
        Self {
            slot: ModuleSlot::default(),
            to_amp: 0.0,
            to_filter_octaves: 0.0,
        }
    }
}

/// A complete synth setup: sound generators -> mixer -> filter -> envelopes
/// -> modulator, with optional model predefinition per slot.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SynthSetup {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub soundgens: Vec<ModuleSlot>,
    #[serde(default)]
    pub mixer: MixerConfig,
    #[serde(default)]
    pub filter: FilterConfig,
    #[serde(default)]
    pub amp_envelope: ModuleSlot,
    #[serde(default)]
    pub filter_envelope: FilterEnvelopeConfig,
    #[serde(default)]
    pub modulator: ModulatorConfig,
}

impl SynthSetup {
    /// The default setup: 4 oscillators/sound generators into the mixer,
    /// then a filter with pregain, amp + filter envelopes with contour, and
    /// a modulator with amounts for amp and filter.
    pub fn default_4osc() -> Self {
        let env_slot = |attack: f32, decay: f32, sustain: f32, release: f32| ModuleSlot {
            model: String::new(),
            params: BTreeMap::from([
                ("attack".to_string(), attack),
                ("decay".to_string(), decay),
                ("sustain".to_string(), sustain),
                ("release".to_string(), release),
            ]),
        };
        Self {
            name: "Default 4-Osc".to_string(),
            mixer: MixerConfig { output_level: 1.0 },
            filter: FilterConfig {
                slot: ModuleSlot {
                    model: String::new(),
                    params: BTreeMap::from([
                        ("filter_type".to_string(), 0.0),
                        ("cutoff".to_string(), 2000.0),
                        ("resonance".to_string(), 0.25),
                    ]),
                },
                pregain_db: 0.0,
            },
            amp_envelope: env_slot(0.01, 0.1, 0.6, 0.2),
            filter_envelope: FilterEnvelopeConfig {
                slot: env_slot(0.02, 0.3, 0.4, 0.4),
                contour_octaves: 2.0,
            },
            modulator: ModulatorConfig {
                slot: ModuleSlot {
                    model: String::new(),
                    params: BTreeMap::from([
                        ("waveform".to_string(), 0.0),
                        ("rate_hz".to_string(), 4.0),
                        ("depth".to_string(), 0.7),
                    ]),
                },
                to_amp: 0.12,
                to_filter_octaves: 1.0,
            },
            soundgens: Vec::new(),
        }
    }

    /// The sound generator slots, falling back to the default 4-osc bank
    /// when the config does not list any.
    pub fn effective_soundgens(&self) -> Vec<ModuleSlot> {
        if !self.soundgens.is_empty() {
            return self.soundgens.clone();
        }
        let slot = |waveform: f32, level: f32, pitch: f32| ModuleSlot {
            model: String::new(),
            params: BTreeMap::from([
                ("waveform".to_string(), waveform),
                ("level".to_string(), level),
                ("pitch_semitones".to_string(), pitch),
            ]),
        };
        vec![
            slot(4.0, 0.25, 0.0),
            slot(0.0, 0.25, 7.0),
            slot(1.0, 0.2, -5.0),
            slot(4.0, 0.18, 12.0),
        ]
    }

    /// Parse a setup from JSON.
    pub fn from_json(source: &str) -> Result<Self, ModuleError> {
        serde_json::from_str(source)
            .map_err(|err| ModuleError::Setup(format!("cannot parse setup: {err}")))
    }

    /// Serialize this setup to pretty JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("setup serialization cannot fail")
    }

    /// Load a setup from a JSON file.
    pub fn load(path: &Path) -> Result<Self, ModuleError> {
        let source = std::fs::read_to_string(path)
            .map_err(|err| ModuleError::Setup(format!("cannot read {}: {err}", path.display())))?;
        Self::from_json(&source)
    }

    /// Save this setup to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), ModuleError> {
        std::fs::write(path, self.to_json())
            .map_err(|err| ModuleError::Setup(format!("cannot write {}: {err}", path.display())))
    }

    /// The per-user directory setups are stored in
    /// (`<data>/Modulus/setups`).
    pub fn setups_dir() -> PathBuf {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("XDG_DATA_HOME").map(PathBuf::from))
            .or_else(|| {
                std::env::var_os("HOME").map(|home| {
                    let home = PathBuf::from(home);
                    if cfg!(target_os = "macos") {
                        home.join("Library/Application Support")
                    } else {
                        home.join(".local/share")
                    }
                })
            })
            .unwrap_or_else(std::env::temp_dir);
        base.join("Modulus").join("setups")
    }

    /// All `*.json` files in a setups directory, sorted by name.
    pub fn scan_dir(dir: &Path) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            })
            .collect();
        files.sort();
        files
    }

    /// Load every valid setup from a directory (unreadable files are
    /// skipped).
    pub fn load_dir(dir: &Path) -> Vec<SynthSetup> {
        Self::scan_dir(dir)
            .iter()
            .filter_map(|path| Self::load(path).ok())
            .collect()
    }

    /// Compile this setup into a routed [`SynthGraph`] of module
    /// instances from `registry`. Modules are looked up by their
    /// (possibly role-defaulted) model name; every slot's category is
    /// validated against its role.
    pub fn build(&self, registry: &ModuleRegistry) -> Result<SynthGraph, ModuleError> {
        let mut sources = Vec::new();
        for (index, slot) in self.effective_soundgens().iter().enumerate() {
            let model = slot.effective_model(DEFAULT_SOUNDGEN_MODEL);
            let mut module = registry.create(&model)?;
            if registry.kind_of(&model) != Some(ModuleKind::SoundGen) {
                return Err(ModuleError::Setup(format!(
                    "soundgen slot {index}: '{model}' is not a sound generator"
                )));
            }
            module.rename(&format!("sg{}", index + 1));
            apply_params(&mut *module, &slot.params);
            sources.push(module);
        }

        let filter_model = self.filter.slot.effective_model(DEFAULT_FILTER_MODEL);
        let mut filter = registry.create(&filter_model)?;
        if registry.kind_of(&filter_model) != Some(ModuleKind::Fx) {
            return Err(ModuleError::Setup(format!(
                "'{filter_model}' is not an FX module"
            )));
        }
        apply_params(&mut *filter, &self.filter.slot.params);

        let amp_model = self.amp_envelope.effective_model(DEFAULT_ENVELOPE_MODEL);
        let mut amp_env = registry.create(&amp_model)?;
        if registry.kind_of(&amp_model) != Some(ModuleKind::Envelope) {
            return Err(ModuleError::Setup(format!(
                "'{amp_model}' is not an envelope module"
            )));
        }
        apply_params(&mut *amp_env, &self.amp_envelope.params);

        let fenv_model = self
            .filter_envelope
            .slot
            .effective_model(DEFAULT_ENVELOPE_MODEL);
        let mut filter_env = registry.create(&fenv_model)?;
        if registry.kind_of(&fenv_model) != Some(ModuleKind::Envelope) {
            return Err(ModuleError::Setup(format!(
                "'{fenv_model}' is not an envelope module"
            )));
        }
        apply_params(&mut *filter_env, &self.filter_envelope.slot.params);

        let mod_model = self.modulator.slot.effective_model(DEFAULT_MODULATOR_MODEL);
        let mut modulator = registry.create(&mod_model)?;
        if registry.kind_of(&mod_model) != Some(ModuleKind::Modulator) {
            return Err(ModuleError::Setup(format!(
                "'{mod_model}' is not a modulator module"
            )));
        }
        apply_params(&mut *modulator, &self.modulator.slot.params);

        let cutoff_base = self
            .filter
            .slot
            .params
            .get("cutoff")
            .copied()
            .unwrap_or(1000.0);

        Ok(SynthGraph {
            name: if self.name.is_empty() {
                "Unnamed Setup".to_string()
            } else {
                self.name.clone()
            },
            sources,
            filter,
            amp_env,
            filter_env,
            modulator,
            cutoff_base,
            pregain_db: self.filter.pregain_db,
            contour_octaves: self.filter_envelope.contour_octaves,
            mod_to_amp: self.modulator.to_amp,
            mod_to_filter_octaves: self.modulator.to_filter_octaves,
            output_level: self.mixer.output_level,
        })
    }
}

/// Set every configured parameter; unknown names are ignored (mirrors the
/// Lua patch engine).
fn apply_params(module: &mut dyn AudioModule, params: &BTreeMap<String, f32>) {
    for (name, value) in params {
        module.set_param(name, *value);
    }
}

/// A compiled setup: module instances wired in the fixed synth topology.
///
/// Rendering is allocation-free and lock-free like the rest of the crate.
pub struct SynthGraph {
    name: String,
    sources: Vec<Box<dyn AudioModule>>,
    filter: Box<dyn AudioModule>,
    amp_env: Box<dyn AudioModule>,
    filter_env: Box<dyn AudioModule>,
    modulator: Box<dyn AudioModule>,
    cutoff_base: f32,
    pregain_db: f32,
    contour_octaves: f32,
    mod_to_amp: f32,
    mod_to_filter_octaves: f32,
    output_level: f32,
}

impl SynthGraph {
    /// The setup name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Number of sound generators in the mixer bus.
    pub fn soundgen_count(&self) -> usize {
        self.sources.len()
    }

    /// Prepare every module for the given sample rate.
    pub fn prepare(&mut self, sample_rate: f32) {
        for source in &mut self.sources {
            source.prepare(sample_rate);
        }
        self.filter.prepare(sample_rate);
        self.amp_env.prepare(sample_rate);
        self.filter_env.prepare(sample_rate);
        self.modulator.prepare(sample_rate);
    }

    /// Reset every module.
    pub fn reset(&mut self) {
        for source in &mut self.sources {
            source.reset();
        }
        self.filter.reset();
        self.amp_env.reset();
        self.filter_env.reset();
        self.modulator.reset();
    }

    /// Forward a note-on to every module.
    pub fn note_on(&mut self, note: u8, velocity: f32, tuning_hz: f32) {
        for source in &mut self.sources {
            source.note_on(note, velocity, tuning_hz);
        }
        self.amp_env.note_on(note, velocity, tuning_hz);
        self.filter_env.note_on(note, velocity, tuning_hz);
        self.modulator.note_on(note, velocity, tuning_hz);
    }

    /// Forward a note-off to every module.
    pub fn note_off(&mut self, note: u8) {
        for source in &mut self.sources {
            source.note_off(note);
        }
        self.amp_env.note_off(note);
        self.filter_env.note_off(note);
        self.modulator.note_off(note);
    }

    /// Render one stereo frame into `frame` (added to whatever is there).
    ///
    /// Routing: sources sum into the mixer bus, pregain drives the bus into
    /// the filter (cutoff modulated by the filter envelope contour and the
    /// modulator), the amp envelope shapes it, the modulator scales it by
    /// its amp amount, and the mixer output level is applied.
    pub fn process_frame(&mut self, frame: &mut [f32; 2], events: &ModuleEvents, sample_rate: f32) {
        let mut bus = [0.0; 2];
        for source in &mut self.sources {
            let mut source_frame = [0.0; 2];
            source.process(&mut source_frame, events, sample_rate);
            bus[0] += source_frame[0];
            bus[1] += source_frame[1];
        }

        // CV-only modules: process a dummy frame to advance their state.
        let mut dummy = [0.0; 2];
        self.filter_env.process(&mut dummy, events, sample_rate);
        let filter_env_cv = self.filter_env.cv();
        self.modulator.process(&mut dummy, events, sample_rate);
        let mod_cv = self.modulator.cv();

        // Pregain is the drive into the filter, not a second master level:
        // it must be applied before the filter stage.
        let pregain = gain_db_to_linear(self.pregain_db);
        bus[0] *= pregain;
        bus[1] *= pregain;

        let cutoff = self.cutoff_base
            * 2.0_f32.powf(
                self.contour_octaves * filter_env_cv + self.mod_to_filter_octaves * (mod_cv - 1.0),
            );
        self.filter
            .set_param("cutoff", cutoff.clamp(20.0, 20_000.0));
        self.filter.process(&mut bus, events, sample_rate);

        self.amp_env.process(&mut bus, events, sample_rate);
        if self.mod_to_amp > 0.0 {
            // The centered LFO CV is 1 +/- depth, so the tremolo dip is
            // symmetric; clamp so the modulation can never boost past unity.
            let amp_mod = (1.0 + (mod_cv - 1.0) * self.mod_to_amp).clamp(0.0, 1.0);
            bus[0] *= amp_mod;
            bus[1] *= amp_mod;
        }

        bus[0] *= self.output_level;
        bus[1] *= self.output_level;

        frame[0] += bus[0];
        frame[1] += bus[1];
    }
}
