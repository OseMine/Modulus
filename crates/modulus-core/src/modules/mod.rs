//! Module engine: a small trait-based API for building audio graphs out of
//! reusable modules (oscillators, filters, envelopes, effects).
//!
//! Modules are plain Rust objects implementing [`AudioModule`]. They are
//! created through a [`ModuleRegistry`](crate::modules::registry::ModuleRegistry),
//! optionally driven by a Lua patch script (feature `lua`) or loaded from a
//! compiled shared library (feature `plugin-host`).
//!
//! # Real-time safety
//!
//! Everything in a [`ModuleGraph`] is boxed once at load time and never
//! allocates again: the render path is allocation-free and lock-free just
//! like the rest of this crate.

#[cfg(feature = "plugin-host")]
pub mod host;
#[cfg(feature = "lua")]
pub mod lua;
pub mod native;
pub mod registry;

use registry::ModuleRegistry;

/// What a module does in the signal chain.
///
/// Categories map 1:1 to the `native/` subfolders and to the ABI kind
/// constants in [`crate::abi`]: `soundgen/` (sound generators), `envelope/`
/// (note-gated amplitude shapers), `modulator/` (free-running modulators)
/// and `fx/` (audio processors).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModuleKind {
    /// A sound generator (synth voice, oscillator, AM bridge): a signal
    /// source whose output is added into the frame.
    SoundGen,
    /// A note-gated amplitude shaper (ADSR, ...): multiplies a gain curve
    /// into the frame.
    Envelope,
    /// A free-running modulation source (LFO, ...): multiplies a slowly
    /// varying signal into the frame.
    Modulator,
    /// An audio processor/effect (filter, chorus, gain, ...): shapes the
    /// frame in place.
    Fx,
}

impl ModuleKind {
    /// Whether the module is a signal source instead of a processor.
    pub fn is_source(self) -> bool {
        matches!(self, ModuleKind::SoundGen)
    }

    /// Human-readable category name (used by docs and editors).
    pub const fn label(self) -> &'static str {
        match self {
            ModuleKind::SoundGen => "Sound Generator",
            ModuleKind::Envelope => "Envelope",
            ModuleKind::Modulator => "Modulator",
            ModuleKind::Fx => "FX",
        }
    }
}

/// Static description of one controllable parameter of a module.
#[derive(Clone, Copy, Debug)]
pub struct ModuleParamSpec {
    pub name: &'static str,
    pub default: f32,
}

/// Per-frame event data handed to every module.
#[derive(Clone, Copy, Debug)]
pub struct ModuleEvents {
    /// Seconds since the graph was prepared.
    pub time_secs: f64,
    /// Current global tuning in Hz.
    pub tuning_hz: f32,
}

/// Errors produced while building, compiling or loading modules.
#[derive(Debug)]
pub enum ModuleError {
    /// Registry lookup failed (unknown module name).
    UnknownModule(String),
    /// A parameter name was not recognized by the module.
    UnknownParam(String),
    /// Loading a dynamic module failed.
    Dynamic(String),
    /// The Lua patch failed to compile or evaluate.
    Lua(String),
    /// A synth setup config failed to parse, build or route.
    Setup(String),
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleError::UnknownModule(name) => write!(f, "unknown module: {name}"),
            ModuleError::UnknownParam(name) => write!(f, "unknown parameter: {name}"),
            ModuleError::Dynamic(msg) => write!(f, "dynamic module error: {msg}"),
            ModuleError::Lua(msg) => write!(f, "lua patch error: {msg}"),
            ModuleError::Setup(msg) => write!(f, "synth setup error: {msg}"),
        }
    }
}

impl std::error::Error for ModuleError {}

/// A single audio module that can be chained inside a [`ModuleGraph`].
///
/// Implementations must be allocation-free inside [`AudioModule::process`],
/// [`AudioModule::prepare`] may allocate.
pub trait AudioModule: Send {
    /// What kind of module this is.
    fn kind(&self) -> ModuleKind;

    /// The instance name used to address this module from patches/hosts.
    fn name(&self) -> &str;

    /// Re-name this instance (used by patch compilers that take an `id`).
    fn rename(&mut self, _id: &str) {}

    /// Set up the module for a new sample rate. May allocate.
    fn prepare(&mut self, sample_rate: f32);

    /// Reset all internal state (voices, delay lines, filters).
    fn reset(&mut self);

    /// Note-on event forwarded from the graph.
    fn note_on(&mut self, _note: u8, _velocity: f32, _tuning_hz: f32) {}

    /// Note-off event forwarded from the graph.
    fn note_off(&mut self, _note: u8) {}

    /// The controllable parameters of this module.
    fn params(&self) -> &[ModuleParamSpec];

    /// Set a parameter by name. Returns `false` if the name is unknown.
    fn set_param(&mut self, name: &str, value: f32) -> bool;

    /// The current value of a parameter, if it exists.
    fn param_value(&self, name: &str) -> Option<f32>;

    /// The module's current modulation/CV value, read after the latest
    /// [`AudioModule::process`] call advanced it.
    ///
    /// `1.0` means "no contribution" and is the default for modules that do
    /// not generate modulation. Envelopes expose their current 0..1 stage
    /// value; modulators expose the gain they currently apply (0..1,
    /// `1 - depth` at the LFO trough, `1.0` at `depth = 0`).
    fn cv(&self) -> f32 {
        1.0
    }

    /// Process one stereo frame. Source modules are handed a zeroed frame
    /// and must write their output into it; processors modify the frame in
    /// place.
    fn process(&mut self, frame: &mut [f32; 2], events: &ModuleEvents, sample_rate: f32);
}

/// An ordered chain of modules that processes stereo frames.
///
/// Sound generators act as sources (their output is added into the frame),
/// envelopes and modulators multiply gain curves into the frame, and FX
/// modules process the frame in order.
pub struct ModuleGraph {
    modules: Vec<Box<dyn AudioModule>>,
}

impl ModuleGraph {
    /// Create a graph from an ordered list of modules.
    pub fn new(modules: Vec<Box<dyn AudioModule>>) -> Self {
        Self { modules }
    }

    /// Prepare every module for the given sample rate.
    pub fn prepare(&mut self, sample_rate: f32) {
        for module in &mut self.modules {
            module.prepare(sample_rate);
        }
    }

    /// Reset every module.
    pub fn reset(&mut self) {
        for module in &mut self.modules {
            module.reset();
        }
    }

    /// Forward a note-on to every module.
    pub fn note_on(&mut self, note: u8, velocity: f32, tuning_hz: f32) {
        for module in &mut self.modules {
            module.note_on(note, velocity, tuning_hz);
        }
    }

    /// Forward a note-off to every module.
    pub fn note_off(&mut self, note: u8) {
        for module in &mut self.modules {
            module.note_off(note);
        }
    }

    /// Process one stereo frame through the whole chain.
    pub fn process_frame(&mut self, frame: &mut [f32; 2], events: &ModuleEvents, sample_rate: f32) {
        for module in &mut self.modules {
            if module.kind().is_source() {
                let mut source_frame = [0.0; 2];
                module.process(&mut source_frame, events, sample_rate);
                frame[0] += source_frame[0];
                frame[1] += source_frame[1];
            } else {
                module.process(frame, events, sample_rate);
            }
        }
    }

    /// The ordered list of module instance names.
    pub fn module_names(&self) -> Vec<&str> {
        self.modules.iter().map(|m| m.name()).collect()
    }

    /// Set a named parameter on a named module instance.
    pub fn set_param(&mut self, module_name: &str, param_name: &str, value: f32) -> bool {
        match self.modules.iter_mut().find(|m| m.name() == module_name) {
            Some(module) => module.set_param(param_name, value),
            None => false,
        }
    }

    /// Number of modules in the graph.
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Whether the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}

/// Convenience: build the built-in registry once.
pub fn builtin_registry() -> ModuleRegistry {
    let mut registry = ModuleRegistry::empty();
    native::register_builtin_modules(&mut registry);
    registry
}
