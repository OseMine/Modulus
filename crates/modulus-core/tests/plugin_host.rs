//! End-to-end test for the compiled module host: loads the `demo-module`
//! shared library and renders audio through it.
//!
//! Requires the demo module to be built first:
//!
//! ```text
//! cargo build -p demo-module
//! ```
//!
//! and the path passed via `MODULUS_DEMO_MODULE`. If the variable is unset
//! the test is skipped.

use std::path::Path;

use modulus_core::modules::host::DynamicModule;
use modulus_core::modules::{AudioModule, ModuleEvents, ModuleKind};

#[test]
fn loads_and_runs_demo_module() {
    let Ok(path) = std::env::var("MODULUS_DEMO_MODULE") else {
        eprintln!(
            "skipping: set MODULUS_DEMO_MODULE to the built demo-module \
             shared library (cargo build -p demo-module)"
        );
        return;
    };

    // SAFETY: demo-module implements the ABI correctly by construction.
    let mut module =
        unsafe { DynamicModule::open(Path::new(&path)) }.expect("demo module should load");

    assert_eq!(module.kind(), ModuleKind::SoundGen);
    assert_eq!(module.name(), "demo");
    assert_eq!(module.params().len(), 3);
    assert!(module.set_param("level", 0.5));
    assert!(!module.set_param("does_not_exist", 1.0));
    assert_eq!(module.param_value("level"), Some(0.5));

    module.prepare(44_100.0);

    let events = ModuleEvents {
        time_secs: 0.0,
        tuning_hz: 440.0,
    };

    let mut frame = [0.0; 2];
    let mut peak = 0.0_f32;
    for _ in 0..4410 {
        module.process(&mut frame, &events, 44_100.0);
        peak = peak.max(frame[0].abs());
        frame[0] = 0.0;
        frame[1] = 0.0;
    }

    // The demo oscillator is free-running; a sine at 440 Hz must be heard.
    assert!(peak > 0.1, "expected audible output, got peak {peak}");
    assert!(peak <= 0.51, "output should be levelled, got {peak}");
}
