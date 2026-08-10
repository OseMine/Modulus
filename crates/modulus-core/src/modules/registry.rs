//! Named registry of module builders.

use std::sync::Arc;

use super::{AudioModule, ModuleError, ModuleKind};

/// Creates a fresh instance of a module.
pub type ModuleBuilder = Arc<dyn Fn() -> Box<dyn AudioModule> + Send + Sync>;

/// A registry that maps module names to builders.
///
/// New modules are added with [`ModuleRegistry::register`]; adding a module
/// is a single function call, so the workspace `variable-*` repos can be
/// dropped in as builders without touching the rest of the codebase.
pub struct ModuleRegistry {
    entries: Vec<(String, ModuleKind, ModuleBuilder)>,
}

impl ModuleRegistry {
    /// An empty registry.
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Register a new module builder under `name`.
    pub fn register(&mut self, name: impl Into<String>, kind: ModuleKind, builder: ModuleBuilder) {
        self.entries.push((name.into(), kind, builder));
    }

    /// Create a module instance by name.
    pub fn create(&self, name: &str) -> Result<Box<dyn AudioModule>, ModuleError> {
        match self.entries.iter().find(|(n, _, _)| n == name) {
            Some((_, _, builder)) => Ok(builder()),
            None => Err(ModuleError::UnknownModule(name.to_string())),
        }
    }

    /// All registered module names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(n, _, _)| n.as_str())
    }

    /// The category of a registered module name, if registered.
    pub fn kind_of(&self, name: &str) -> Option<ModuleKind> {
        self.entries
            .iter()
            .find(|(n, _, _)| n == name)
            .map(|(_, k, _)| *k)
    }

    /// All registered module names in one category (e.g. every `SoundGen`).
    pub fn names_by_kind(&self, kind: ModuleKind) -> impl Iterator<Item = &str> {
        self.entries
            .iter()
            .filter(move |(_, k, _)| *k == kind)
            .map(|(n, _, _)| n.as_str())
    }
}
