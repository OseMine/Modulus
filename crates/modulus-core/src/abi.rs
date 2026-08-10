//! Stable C ABI for compiled Modulus modules.
//!
//! Any language that can produce a C ABI (Rust, C, C++, Python via C
//! extensions) can implement this interface and be loaded at runtime by
//! [`crate::modules::host::DynamicModule`].
//!
//! # Safety
//!
//! Every function below is `unsafe extern "C"` and must not panic or unwind.

#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_void};

/// FourCC `'MODU'` at the start of [`ModulusModuleInfo`].
pub const MODULUS_MODULE_MAGIC: u32 = 0x4D4F_4455;
/// Bump when incompatible ABI changes are made.
pub const MODULUS_API_VERSION: u32 = 1;

pub const MODULUS_KIND_OSCILLATOR: u32 = 0;
pub const MODULUS_KIND_FILTER: u32 = 1;
pub const MODULUS_KIND_ENVELOPE: u32 = 2;
pub const MODULUS_KIND_EFFECT: u32 = 3;

/// Module descriptor returned by `modulus_module_info`.
#[repr(C)]
pub struct ModulusModuleInfo {
    pub magic: u32,
    pub api_version: u32,
    pub kind: u32,
    pub param_count: u32,
    /// Static, NUL-terminated module name.
    pub name: *const c_char,
    /// `param_count` static, NUL-terminated parameter names.
    pub param_names: *const *const c_char,
    /// `param_count` default parameter values.
    pub param_defaults: *const f32,
}

// The descriptor is read-only static data provided by the library; the
// pointers are only dereferenced while the library is loaded.
unsafe impl Send for ModulusModuleInfo {}
unsafe impl Sync for ModulusModuleInfo {}

/// The full set of exports a compiled module must provide.
pub mod exports {
    use super::*;

    /// Returns a pointer to a static [`ModulusModuleInfo`].
    pub type ModulusModuleInfoFn = unsafe extern "C" fn() -> *const ModulusModuleInfo;
    /// Creates a module instance.
    pub type ModulusModuleCreateFn = unsafe extern "C" fn() -> *mut c_void;
    /// Destroys a module instance.
    pub type ModulusModuleDestroyFn = unsafe extern "C" fn(*mut c_void);
    /// Prepares a module for a sample rate.
    pub type ModulusModulePrepareFn = unsafe extern "C" fn(*mut c_void, f32);
    /// Resets a module.
    pub type ModulusModuleResetFn = unsafe extern "C" fn(*mut c_void);
    /// Processes one stereo frame: `in_l` and `in_r` point at single samples.
    pub type ModulusModuleProcessFn = unsafe extern "C" fn(
        *mut c_void,
        *mut f32,
        *mut f32,
        *const f32,
        f32,
    );
}

pub const SYMBOL_INFO: &[u8] = b"modulus_module_info\0";
pub const SYMBOL_CREATE: &[u8] = b"modulus_module_create\0";
pub const SYMBOL_DESTROY: &[u8] = b"modulus_module_destroy\0";
pub const SYMBOL_PREPARE: &[u8] = b"modulus_module_prepare\0";
pub const SYMBOL_RESET: &[u8] = b"modulus_module_reset\0";
pub const SYMBOL_PROCESS: &[u8] = b"modulus_module_process\0";