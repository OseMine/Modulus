//! Tests for module categorization and the Am-Synth bridge + LFO modules.

use modulus_core::modules::{builtin_registry, AudioModule, ModuleEvents, ModuleKind};

fn events() -> ModuleEvents {
    ModuleEvents {
        note_on: None,
        note_off: None,
        time_secs: 0.0,
        tuning_hz: 440.0,
    }
}

#[test]
fn builtin_modules_are_categorized() {
    let registry = builtin_registry();
    assert_eq!(registry.kind_of("oscillator"), Some(ModuleKind::SoundGen));
    assert_eq!(registry.kind_of("oscillator2"), Some(ModuleKind::SoundGen));
    assert_eq!(registry.kind_of("am_bridge"), Some(ModuleKind::SoundGen));
    assert_eq!(registry.kind_of("envelope"), Some(ModuleKind::Envelope));
    assert_eq!(registry.kind_of("lfo"), Some(ModuleKind::Modulator));
    assert_eq!(registry.kind_of("filter"), Some(ModuleKind::Fx));
    assert_eq!(registry.kind_of("chorus"), Some(ModuleKind::Fx));
    assert_eq!(registry.kind_of("gain"), Some(ModuleKind::Fx));
}

#[test]
fn names_by_kind_groups_registry() {
    let registry = builtin_registry();
    let mut soundgen: Vec<&str> = registry.names_by_kind(ModuleKind::SoundGen).collect();
    soundgen.sort_unstable();
    assert_eq!(soundgen, vec!["am_bridge", "oscillator", "oscillator2"]);

    let envelopes: Vec<&str> = registry.names_by_kind(ModuleKind::Envelope).collect();
    assert_eq!(envelopes, vec!["envelope"]);

    let modulators: Vec<&str> = registry.names_by_kind(ModuleKind::Modulator).collect();
    assert_eq!(modulators, vec!["lfo"]);

    let mut fx: Vec<&str> = registry.names_by_kind(ModuleKind::Fx).collect();
    fx.sort_unstable();
    assert_eq!(fx, vec!["chorus", "filter", "gain"]);
}

fn render(module: &mut Box<dyn AudioModule>, samples: usize, gate: bool) -> (f32, f32) {
    module.prepare(44_100.0);
    let mut frame = [0.0; 2];
    let mut peak = 0.0_f32;
    let mut min = f32::MAX;
    for sample in 0..samples {
        if sample == 0 && gate {
            module.note_on(60, 1.0, 440.0);
        }
        module.process(&mut frame, &events(), 44_100.0);
        peak = peak.max(frame[0].abs());
        min = min.min(frame[0]);
    }
    (peak, min)
}

#[test]
fn am_bridge_mix_mode_sums_carrier_and_modulator() {
    let registry = builtin_registry();
    let mut bridge = registry.create("am_bridge").unwrap();
    assert_eq!(bridge.kind(), ModuleKind::SoundGen);
    assert!(bridge.set_param("mode", 0.0));
    // Two phase-locked sine oscillators at level 1 sum to twice the signal.
    assert!(bridge.set_param("carrier_level", 1.0));
    assert!(bridge.set_param("modulator_level", 1.0));
    assert!(bridge.set_param("carrier_waveform", 0.0));
    assert!(bridge.set_param("modulator_waveform", 0.0));
    assert!(bridge.set_param("carrier_pitch", 0.0));
    assert!(bridge.set_param("modulator_pitch", 0.0));

    let (peak, _) = render(&mut bridge, 2048, true);
    assert!(
        peak > 1.9,
        "mix mode should sum both oscillators, peak {peak}"
    );
}

#[test]
fn am_bridge_am_mode_depth_zero_is_carrier_only() {
    let registry = builtin_registry();
    let mut bridge = registry.create("am_bridge").unwrap();
    assert!(bridge.set_param("mode", 1.0));
    assert!(bridge.set_param("am_depth", 0.0));
    assert!(bridge.set_param("carrier_level", 1.0));

    let (peak, _) = render(&mut bridge, 4096, true);
    // Pure carrier at level 1: sine stays within [-1, 1].
    assert!(
        peak > 0.9 && peak <= 1.0,
        "depth 0 should be carrier only, peak {peak}"
    );
}

#[test]
fn am_bridge_am_depth_changes_the_signal() {
    let registry = builtin_registry();
    let render_depth = |depth: f32| {
        let mut bridge = registry.create("am_bridge").unwrap();
        assert!(bridge.set_param("mode", 1.0));
        assert!(bridge.set_param("am_depth", depth));
        assert!(bridge.set_param("carrier_level", 1.0));
        let (peak, _) = render(&mut bridge, 8192, true);
        peak
    };
    let shallow = render_depth(0.1);
    let deep = render_depth(1.0);
    assert!(
        (deep - shallow).abs() > 0.05,
        "AM depth should change the output: shallow {shallow}, deep {deep}"
    );
}

#[test]
fn am_bridge_is_silent_without_gate() {
    let registry = builtin_registry();
    let mut bridge = registry.create("am_bridge").unwrap();
    let (peak, _) = render(&mut bridge, 1024, false);
    assert_eq!(peak, 0.0);
}

#[test]
fn lfo_is_a_passthrough_at_zero_depth() {
    let registry = builtin_registry();
    let mut lfo = registry.create("lfo").unwrap();
    assert_eq!(lfo.kind(), ModuleKind::Modulator);
    assert!(lfo.set_param("depth", 0.0));
    lfo.prepare(44_100.0);

    let mut frame = [0.25; 2];
    lfo.process(&mut frame, &events(), 44_100.0);
    assert_eq!(frame[0], 0.25, "depth 0 must be an exact passthrough");
    assert_eq!(frame[1], 0.25);
}

#[test]
fn lfo_modulates_over_time() {
    let registry = builtin_registry();
    let mut lfo = registry.create("lfo").unwrap();
    assert!(lfo.set_param("rate_hz", 2.0));
    assert!(lfo.set_param("depth", 1.0));
    lfo.prepare(44_100.0);

    let mut min = f32::MAX;
    let mut max = f32::MIN;
    // Fresh unity frame every sample: measures the *gain* curve, not the
    // accumulated product.
    for _ in 0..(44_100 / 2) {
        let mut frame = [1.0; 2];
        lfo.process(&mut frame, &events(), 44_100.0);
        min = min.min(frame[0]);
        max = max.max(frame[0]);
    }
    // Full-depth sine LFO swells between silence and unity.
    assert!(min < 1e-3, "expected silence at the LFO trough, got {min}");
    assert!((max - 1.0).abs() < 1e-3, "expected unity peak, got {max}");
}

#[test]
fn lfo_ignores_note_events() {
    let registry = builtin_registry();
    let mut lfo = registry.create("lfo").unwrap();
    assert!(lfo.set_param("depth", 1.0));
    assert!(lfo.set_param("rate_hz", 20.0));
    lfo.prepare(44_100.0);
    lfo.note_on(60, 1.0, 440.0);

    let mut changed = false;
    let mut prev = 0.0_f32;
    for _ in 0..16 {
        let mut frame = [1.0; 2];
        lfo.process(&mut frame, &events(), 44_100.0);
        if prev != 0.0 && (prev - frame[0]).abs() > 1e-3 {
            changed = true;
        }
        prev = frame[0];
    }
    assert!(changed, "LFO should keep running independently of gates");
}
