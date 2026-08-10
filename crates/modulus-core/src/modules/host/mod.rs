//! Runtime host for compiled Modulus modules (feature `plugin-host`).
//!
//! Loads a shared library (`.dll`/`.dylib`/`.so`) implementing the module
//! C ABI in [`crate::abi`] and exposes it as a regular [`AudioModule`].
//! Loading happens once at setup time, never inside the audio callback.

pub use crate::abi::{
    ModulusModuleInfo, MODULUS_API_VERSION, MODULUS_KIND_ENVELOPE, MODULUS_KIND_FX,
    MODULUS_KIND_MODULATOR, MODULUS_KIND_SOUNDGEN, MODULUS_MODULE_MAGIC,
};

use std::ffi::CStr;
use std::os::raw::c_void;
use std::path::Path;

use libloading::Library;

use super::{AudioModule, ModuleError, ModuleEvents, ModuleKind, ModuleParamSpec};

/// A compiled module loaded from a shared library.
pub struct DynamicModule {
    /// Kept alive for the lifetime of the module (function pointers stay valid).
    #[allow(dead_code)]
    lib: Library,
    handle: *mut c_void,
    kind: ModuleKind,
    name: String,
    param_specs: Vec<ModuleParamSpec>,
    param_values: Vec<f32>,
    destroy: crate::abi::exports::ModulusModuleDestroyFn,
    prepare: crate::abi::exports::ModulusModulePrepareFn,
    reset: crate::abi::exports::ModulusModuleResetFn,
    process: crate::abi::exports::ModulusModuleProcessFn,
}

unsafe impl Send for DynamicModule {}

impl DynamicModule {
    /// Load and validate a compiled module from a shared library path.
    ///
    /// # Safety
    ///
    /// The library must export the Modulus module ABI symbols and must
    /// implement them correctly; this is a trust boundary.
    pub unsafe fn open(path: &Path) -> Result<Self, ModuleError> {
        let lib = Library::new(path)
            .map_err(|err| ModuleError::Dynamic(err.to_string()))?;

        let info_fn: libloading::Symbol<'_, crate::abi::exports::ModulusModuleInfoFn> = lib
            .get(crate::abi::SYMBOL_INFO)
            .map_err(|err| ModuleError::Dynamic(format!("missing info symbol: {err}")))?;
        let info_ptr = info_fn();
        let info = &*info_ptr;
        if info.magic != MODULUS_MODULE_MAGIC {
            return Err(ModuleError::Dynamic("bad module magic".into()));
        }
        if info.api_version != MODULUS_API_VERSION {
            return Err(ModuleError::Dynamic(format!(
                "unsupported API version {}",
                info.api_version
            )));
        }

        let name = if info.name.is_null() {
            String::new()
        } else {
            CStr::from_ptr(info.name)
                .to_str()
                .map_err(|_| ModuleError::Dynamic("non-utf8 module name".into()))?
                .to_string()
        };

let kind = match info.kind {
            MODULUS_KIND_SOUNDGEN => ModuleKind::SoundGen,
            MODULUS_KIND_ENVELOPE => ModuleKind::Envelope,
            MODULUS_KIND_MODULATOR => ModuleKind::Modulator,
            MODULUS_KIND_FX => ModuleKind::Fx,
            other => {
                return Err(ModuleError::Dynamic(format!(
                    "unknown module kind {other}"
                )))
            }
        };

        let mut param_specs = Vec::with_capacity(info.param_count as usize);
        let mut param_values = Vec::with_capacity(info.param_count as usize);
        for index in 0..info.param_count as usize {
            let param_name = CStr::from_ptr(*info.param_names.add(index))
                .to_str()
                .map_err(|_| ModuleError::Dynamic("non-utf8 param name".into()))?
                .to_string();
            param_specs.push(ModuleParamSpec {
                name: Box::leak(param_name.into_boxed_str()),
                default: *info.param_defaults.add(index),
            });
            param_values.push(*info.param_defaults.add(index));
        }

        let create: crate::abi::exports::ModulusModuleCreateFn = *lib
            .get(crate::abi::SYMBOL_CREATE)
            .map_err(|err| ModuleError::Dynamic(format!("missing create symbol: {err}")))?;
        let handle = create();

        let destroy: crate::abi::exports::ModulusModuleDestroyFn = *lib
            .get(crate::abi::SYMBOL_DESTROY)
            .map_err(|err| ModuleError::Dynamic(format!("missing destroy symbol: {err}")))?;
        let prepare: crate::abi::exports::ModulusModulePrepareFn = *lib
            .get(crate::abi::SYMBOL_PREPARE)
            .map_err(|err| ModuleError::Dynamic(format!("missing prepare symbol: {err}")))?;
        let reset: crate::abi::exports::ModulusModuleResetFn = *lib
            .get(crate::abi::SYMBOL_RESET)
            .map_err(|err| ModuleError::Dynamic(format!("missing reset symbol: {err}")))?;
        let process: crate::abi::exports::ModulusModuleProcessFn = *lib
            .get(crate::abi::SYMBOL_PROCESS)
            .map_err(|err| ModuleError::Dynamic(format!("missing process symbol: {err}")))?;

        Ok(Self {
            lib,
            handle,
            kind,
            name,
            param_specs,
            param_values,
            destroy,
            prepare,
            reset,
            process,
        })
    }

    /// The declared kind of the loaded module.
    pub fn kind(&self) -> ModuleKind {
        self.kind
    }
}

impl Drop for DynamicModule {
    fn drop(&mut self) {
        // SAFETY: handle was created by this module's create symbol.
        unsafe { (self.destroy)(self.handle) }
    }
}

impl AudioModule for DynamicModule {
    fn kind(&self) -> ModuleKind {
        self.kind
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn prepare(&mut self, sample_rate: f32) {
        // SAFETY: handle is valid and the ABI promises no panics.
        unsafe { (self.prepare)(self.handle, sample_rate) }
    }

    fn reset(&mut self) {
        // SAFETY: handle is valid and the ABI promises no panics.
        unsafe { (self.reset)(self.handle) }
    }

    fn params(&self) -> &[ModuleParamSpec] {
        &self.param_specs
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match self
            .param_specs
            .iter()
            .position(|spec| spec.name == name)
        {
            Some(index) => {
                self.param_values[index] = value;
                true
            }
            None => false,
        }
    }

    fn param_value(&self, name: &str) -> Option<f32> {
        self.param_specs
            .iter()
            .position(|spec| spec.name == name)
            .map(|index| self.param_values[index])
    }

    fn process(&mut self, frame: &mut [f32; 2], _events: &ModuleEvents, sample_rate: f32) {
        // SAFETY: handle is valid, buffers point at live locals, the ABI
        // promises no panics.
        unsafe {
            (self.process)(
                self.handle,
                &mut frame[0] as *mut f32,
                &mut frame[1] as *mut f32,
                self.param_values.as_ptr(),
                sample_rate,
            )
        }
    }
}

