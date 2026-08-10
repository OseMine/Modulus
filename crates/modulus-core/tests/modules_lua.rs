//! End-to-end tests for the Lua patch engine and the native module graph.

use modulus_core::modules::{builtin_registry, lua, ModuleEvents};

const PATCH: &str = r#"
return {
  name = "Test Patch",
  modules = {
    { kind = "oscillator", id = "osc1", waveform = 4, level = 0.5, pitch_semitones = 0 },
    { kind = "filter",     id = "filt", filter_type = 0, cutoff = 1500, resonance = 0.2 },
    { kind = "envelope",   id = "env",  attack = 0.01, decay = 0.05, sustain = 0.7, release = 0.05 },
    { kind = "gain",       id = "out",  gain_db = -6 },
  },
}
"#;

fn events() -> ModuleEvents {
    ModuleEvents {
        note_on: None,
        note_off: None,
        time_secs: 0.0,
        tuning_hz: 440.0,
    }
}

#[test]
fn builds_graph_from_lua_patch() {
    let registry = builtin_registry();
    let graph = lua::build_patch(&registry, PATCH).expect("patch should compile");

    assert_eq!(graph.len(), 4);
    let names = graph.module_names();
    assert!(names.contains(&"osc1"));
    assert!(names.contains(&"filt"));
    assert!(names.contains(&"env"));
    assert!(names.contains(&"out"));
}

#[test]
fn renders_audible_signal_with_envelope() {
    let registry = builtin_registry();
    let mut graph = lua::build_patch(&registry, PATCH).expect("patch should compile");
    graph.prepare(44_100.0);

    let mut frame = [0.0; 2];
    graph.note_on(60, 1.0, 440.0);
    let mut peak = 0.0_f32;
    for _ in 0..2048 {
        graph.process_frame(&mut frame, &events(), 44_100.0);
        peak = peak.max(frame[0].abs());
    }
    // Envelope attacks to ~0.7, gain -6 dB => ~0.35 peak. Definitely audible.
    assert!(peak > 0.1, "expected audible output, got peak {peak}");
}

#[test]
fn gate_off_silences_the_graph() {
    let registry = builtin_registry();
    let mut graph = lua::build_patch(&registry, PATCH).expect("patch should compile");
    graph.prepare(44_100.0);

    let mut frame = [0.0; 2];
    // Never pressed a note - the oscillator is gated off.
    for _ in 0..1024 {
        graph.process_frame(&mut frame, &events(), 44_100.0);
    }
    assert_eq!(frame[0], 0.0);
    assert_eq!(frame[1], 0.0);
}

#[test]
fn filter_shape_changes_audibly() {
    let registry = builtin_registry();
    let mut graph = lua::build_patch(&registry, PATCH).expect("patch should compile");
    graph.prepare(44_100.0);
    graph.note_on(60, 1.0, 440.0);

    let measure = |graph: &mut modulus_core::modules::ModuleGraph| {
        let mut frame = [0.0; 2];
        let mut peak = 0.0_f32;
        for _ in 0..4096 {
            graph.process_frame(&mut frame, &events(), 44_100.0);
            peak = peak.max(frame[0].abs());
        }
        peak
    };

    let open = measure(&mut graph);
    graph.set_param("filt", "cutoff", 100.0);
    let closed = measure(&mut graph);
    assert!(
        closed < open,
        "closing the filter should reduce output: open {open}, closed {closed}"
    );
}

#[test]
fn unknown_module_is_rejected() {
    let registry = builtin_registry();
    let bad = "return { modules = { { kind = \"theremin\" } } }";
    let result = lua::build_patch(&registry, bad);
    match result {
        Err(modulus_core::modules::ModuleError::UnknownModule(name)) => {
            assert_eq!(name, "theremin")
        }
        Ok(_) => panic!("expected UnknownModule error, but patch compiled"),
        Err(other) => panic!("expected UnknownModule error, got {other:?}"),
    }
}

#[test]
fn lua_patch_with_bridge_and_lfo_compiles() {
    let registry = builtin_registry();
    let bridge_patch = r#"
return {
  name = "Bridge Test",
  modules = {
    { kind = "am_bridge", id = "bridge", carrier_waveform = 4, am_depth = 0.8 },
    { kind = "lfo", id = "trem", rate_hz = 5, depth = 0.3 },
    { kind = "envelope", id = "env" },
    { kind = "gain", id = "out", gain_db = -12 },
  },
}
"#;
    let mut graph = lua::build_patch(&registry, bridge_patch).expect("patch should compile");
    assert_eq!(graph.len(), 4);
    assert!(graph.module_names().contains(&"bridge"));
    assert!(graph.set_param("bridge", "am_depth", 0.9));
    assert!(graph.set_param("trem", "depth", 0.5));
    assert!(!graph.set_param("bridge", "nope", 0.0));
}
