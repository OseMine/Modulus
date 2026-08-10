//! Example compiled Modulus module.
//!
//! Exports the Modulus module C ABI (`modulus_module_*`) as a straightforward
//! oscillator, so the runtime loader in `modulus-core::modules::host` can be
//! exercised end-to-end. Any language that can produce a C ABI (Rust, C,
//! C++, Python via C extensions) can implement the same contract.

use std::os::raw::{c_char, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use modulus_core::abi::{
    ModulusModuleInfo, MODULUS_API_VERSION, MODULUS_KIND_OSCILLATOR, MODULUS_MODULE_MAGIC,
};

const PARAM_COUNT: u32 = 3;
const PARAM_DEFAULTS: [f32; PARAM_COUNT as usize] = [0.7, 0.0, 0.0];

// Raw C string storage for the info struct (lives for the program duration).
static NAME_CSTR: [u8; 5] = *b"demo\0";

/// `[*const c_char; N]` is not `Sync`; the array is immutable static data.
#[repr(transparent)]
struct NameArray([*const c_char; PARAM_COUNT as usize]);
unsafe impl Sync for NameArray {}

static PARAM_NAMES_C: NameArray = NameArray([
    c"level".as_ptr(),
    c"waveform".as_ptr(),
    c"pitch_semitones".as_ptr(),
]);

static INFO: ModulusModuleInfo = ModulusModuleInfo {
    magic: MODULUS_MODULE_MAGIC,
    api_version: MODULUS_API_VERSION,
    kind: MODULUS_KIND_OSCILLATOR,
    param_count: PARAM_COUNT,
    name: NAME_CSTR.as_ptr() as *const c_char,
    param_names: PARAM_NAMES_C.0.as_ptr(),
    param_defaults: PARAM_DEFAULTS.as_ptr(),
};

static PANIC_FLAG: AtomicBool = AtomicBool::new(false);

/// A stateful demo oscillator (linear-phase sine).
struct DemoOsc {
    phase: f32,
    phase_increment: f32,
    level: f32,
    pitch_semitones: f32,
}

impl DemoOsc {
    fn new() -> Self {
        Self {
            phase: 0.0,
            phase_increment: 0.0,
            level: PARAM_DEFAULTS[0],
            pitch_semitones: PARAM_DEFAULTS[2],
        }
    }
}

/// Sets a panic flag and swallows the panic message; the next ABI call
/// reports the failure instead of unwinding across the FFI boundary.
fn catch_panic<F: FnOnce() -> R + std::panic::UnwindSafe, R>(f: F) -> Option<R> {
    match std::panic::catch_unwind(f) {
        Ok(value) => Some(value),
        Err(_) => {
            PANIC_FLAG.store(true, Ordering::Relaxed);
            None
        }
    }
}

#[no_mangle]
pub extern "C" fn modulus_module_info() -> *const ModulusModuleInfo {
    &INFO as *const ModulusModuleInfo
}

#[no_mangle]
pub extern "C" fn modulus_module_create() -> *mut c_void {
    match catch_panic(|| Box::into_raw(Box::new(DemoOsc::new())) as *mut c_void) {
        Some(ptr) => ptr,
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
/// # Safety
///
/// `module` must be a handle returned by `modulus_module_create` that has not
/// been destroyed yet.
pub unsafe extern "C" fn modulus_module_destroy(module: *mut c_void) {
    let _ = catch_panic(|| {
        if !module.is_null() {
            drop(Box::from_raw(module as *mut DemoOsc));
        }
    });
}

#[no_mangle]
/// # Safety
///
/// `module` must be a live handle returned by `modulus_module_create`.
pub unsafe extern "C" fn modulus_module_prepare(module: *mut c_void, sample_rate: f32) {
    let _ = catch_panic(|| {
        if sample_rate > 0.0 {
            let osc = &mut *(module as *mut DemoOsc);
            osc.update_phase_increment(sample_rate, 440.0);
        }
    });
}

#[no_mangle]
/// # Safety
///
/// `module` must be a live handle returned by `modulus_module_create`.
pub unsafe extern "C" fn modulus_module_reset(module: *mut c_void) {
    let _ = catch_panic(|| {
        (*module.cast::<DemoOsc>()).phase = 0.0;
    });
}

#[no_mangle]
/// # Safety
///
/// `module` must be a live handle returned by `modulus_module_create`;
/// `in_l`/`in_r` must point at writable single samples; `params` must point
/// at `param_count` readable floats.
pub unsafe extern "C" fn modulus_module_process(
    module: *mut c_void,
    in_l: *mut f32,
    in_r: *mut f32,
    params: *const f32,
    _sample_rate: f32,
) {
    let _ = catch_panic(|| {
        let osc = &mut *module.cast::<DemoOsc>();
        // Keep the first param slot free for re-tuning via `params[0]`-style
        // automation; demo uses all three declared params.
        osc.level = *params.add(0);
        osc.pitch_semitones = *params.add(2);
        let sample = osc.sample();
        *in_l = sample;
        *in_r = sample;
    });
}

impl DemoOsc {
    fn update_phase_increment(&mut self, sample_rate: f32, frequency: f32) {
        let semitones = 2.0_f32.powf(self.pitch_semitones / 12.0);
        self.phase_increment =
            std::f32::consts::TAU * frequency * semitones / sample_rate.max(1.0);
    }

    fn sample(&mut self) -> f32 {
        let value = self.phase.sin() * self.level;
        self.phase += self.phase_increment;
        if self.phase >= std::f32::consts::TAU {
            self.phase -= std::f32::consts::TAU;
        }
        value
    }
}