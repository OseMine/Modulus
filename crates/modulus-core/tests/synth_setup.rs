//! Tests for the JSON synth setup configs and the routed SynthGraph.

use std::path::PathBuf;

use modulus_core::modules::builtin_registry;
use modulus_core::modules::native;
use modulus_core::modules::registry::ModuleRegistry;
use modulus_core::modules::ModuleEvents;
use modulus_core::synth_setup::SynthSetup;

const DEFAULT_JSON: &str = include_str!("../../../setups/default.json");
const BRIDGE_JSON: &str = include_str!("../../../setups/bridge_lead.json");

fn events() -> ModuleEvents {
    ModuleEvents {
        note_on: None,
        note_off: None,
        time_secs: 0.0,
        tuning_hz: 440.0,
    }
}

fn render(graph: &mut modulus_core::synth_setup::SynthGraph, samples: usize) -> f32 {
    graph.prepare(44_100.0);
    graph.reset();
    graph.note_on(60, 1.0, 440.0);
    let mut frame = [0.0; 2];
    let mut peak = 0.0_f32;
    for _ in 0..samples {
        graph.process_frame(&mut frame, &events(), 44_100.0);
        peak = peak.max(frame[0].abs()).max(frame[1].abs());
        frame = [0.0; 2];
    }
    peak
}

#[test]
fn default_json_parses_builds_and_renders() {
    let setup = SynthSetup::from_json(DEFAULT_JSON).expect("default.json should parse");
    assert_eq!(setup.name, "Default 4-Osc");
    let mut graph = setup.build(&builtin_registry()).expect("should build");
    assert_eq!(graph.soundgen_count(), 4);
    let peak = render(&mut graph, 4096);
    assert!(peak > 0.05, "expected audible output, got peak {peak}");
}

#[test]
fn default_config_has_four_soundgens() {
    let setup = SynthSetup::default_4osc();
    assert_eq!(setup.effective_soundgens().len(), 4);
    assert_eq!(setup.name, "Default 4-Osc");
}

#[test]
fn json_roundtrip_is_stable() {
    let setup = SynthSetup::from_json(DEFAULT_JSON).unwrap();
    let reencoded = SynthSetup::from_json(&setup.to_json()).unwrap();
    assert_eq!(setup, reencoded);
}

#[test]
fn model_predefinition_is_optional() {
    // A registry without the default "oscillator" model.
    let mut trimmed = ModuleRegistry::empty();
    native::soundgen::am_bridge::register(&mut trimmed);
    native::fx::filter::register(&mut trimmed);
    native::envelope::adsr::register(&mut trimmed);
    native::modulator::lfo::register(&mut trimmed);

    // A fully-predefined setup builds even without the default models.
    let predefined = r#"
    {
      "soundgens": [ { "model": "am_bridge" }, { "model": "am_bridge" } ],
      "filter":         { "model": "filter" },
      "amp_envelope":   { "model": "envelope" },
      "filter_envelope": { "model": "envelope" },
      "modulator":      { "model": "lfo" }
    }
    "#;
    let setup = SynthSetup::from_json(predefined).unwrap();
    let mut graph = setup
        .build(&trimmed)
        .expect("predefined setup should build");
    assert_eq!(graph.soundgen_count(), 2);
    let peak = render(&mut graph, 4096);
    assert!(peak > 0.05, "expected audible output, got peak {peak}");

    // The default setup relies on "oscillator", which is missing here.
    assert!(SynthSetup::default_4osc().build(&trimmed).is_err());

    // bridge_lead.json predfinines its models (slot 2 keeps the default);
    // it builds and renders on the full registry.
    let bridge = SynthSetup::from_json(BRIDGE_JSON).unwrap();
    let mut graph = bridge.build(&builtin_registry()).unwrap();
    assert_eq!(graph.soundgen_count(), 2);
    let peak = render(&mut graph, 4096);
    assert!(peak > 0.05, "expected audible output, got peak {peak}");
}

#[test]
fn unknown_model_is_rejected() {
    let setup = SynthSetup::from_json(r#"{ "soundgens": [ { "model": "theremin" } ] }"#).unwrap();
    match setup.build(&builtin_registry()) {
        Err(modulus_core::modules::ModuleError::UnknownModule(name)) => {
            assert_eq!(name, "theremin")
        }
        Ok(_) => panic!("expected UnknownModule error, but setup built"),
        Err(other) => panic!("expected UnknownModule error, got {other:?}"),
    }
}

#[test]
fn wrong_module_category_is_rejected() {
    // An LFO cannot fill the sound generator role.
    let setup = SynthSetup::from_json(r#"{ "soundgens": [ { "model": "lfo" } ] }"#).unwrap();
    match setup.build(&builtin_registry()) {
        Err(modulus_core::modules::ModuleError::Setup(msg)) => {
            assert!(msg.contains("sound generator"), "got: {msg}")
        }
        Ok(_) => panic!("expected Setup error, but setup built"),
        Err(other) => panic!("expected Setup error, got {other:?}"),
    }
}

#[test]
fn invalid_json_is_rejected() {
    assert!(SynthSetup::from_json("not json at all").is_err());
}

#[test]
fn amp_modulation_changes_output() {
    let render_with = |to_amp: f32| {
        let mut setup = SynthSetup::from_json(DEFAULT_JSON).unwrap();
        setup.modulator.slot.params.insert("depth".into(), 1.0);
        setup.modulator.to_amp = to_amp;
        let mut graph = setup.build(&builtin_registry()).unwrap();
        // Half a second at rate_hz 4: the LFO swings fully several times.
        render(&mut graph, 22_050)
    };
    let none = render_with(0.0);
    let tremolo = render_with(1.0);
    assert!(
        (tremolo - none).abs() > 0.05,
        "amp modulation should change output: none {none}, tremolo {tremolo}"
    );
}

#[test]
fn filter_contour_sweeps_cutoff() {
    let render_with = |contour: f32| {
        let mut setup = SynthSetup::from_json(DEFAULT_JSON).unwrap();
        setup.filter_envelope.contour_octaves = contour;
        let mut graph = setup.build(&builtin_registry()).unwrap();
        // During the first samples the filter envelope attacks toward 1,
        // so a positive contour opens the (saw-heavy) mix louder.
        render(&mut graph, 4096)
    };
    let flat = render_with(0.0);
    let swept = render_with(3.0);
    assert!(
        swept > flat,
        "contour should raise the cutoff during attack: flat {flat}, swept {swept}"
    );
}

#[test]
fn note_off_releases_the_envelopes() {
    let mut setup = SynthSetup::from_json(DEFAULT_JSON).unwrap();
    setup.amp_envelope.params.insert("release".into(), 0.05);
    let mut graph = setup.build(&builtin_registry()).unwrap();
    graph.prepare(44_100.0);
    graph.note_on(60, 1.0, 440.0);

    let mut frame = [0.0; 2];
    let mut attack_peak = 0.0_f32;
    for _ in 0..2048 {
        graph.process_frame(&mut frame, &events(), 44_100.0);
        attack_peak = attack_peak.max(frame[0].abs());
        frame = [0.0; 2];
    }
    graph.note_off(60);
    // Skip the audible release tail (0.05s), then measure the silence after
    // the envelopes have fully closed.
    let mut dummy = [0.0; 2];
    for _ in 0..4410 {
        graph.process_frame(&mut dummy, &events(), 44_100.0);
        dummy = [0.0; 2];
    }
    let mut tail_peak = 0.0_f32;
    for _ in 0..22_050 {
        graph.process_frame(&mut frame, &events(), 44_100.0);
        tail_peak = tail_peak.max(frame[0].abs());
        frame = [0.0; 2];
    }
    assert!(
        tail_peak < 1e-4,
        "release should silence the setup: attack {attack_peak}, tail {tail_peak}"
    );
}

#[test]
fn scan_and_load_dir_find_all_setups() {
    let dir = std::env::temp_dir().join(format!("modulus-setup-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a_default.json"), DEFAULT_JSON).unwrap();
    std::fs::write(dir.join("b_bridge.json"), BRIDGE_JSON).unwrap();
    std::fs::write(dir.join("junk.txt"), "nope").unwrap();

    let files = SynthSetup::scan_dir(&dir);
    assert_eq!(files.len(), 2);
    let setups = SynthSetup::load_dir(&dir);
    assert_eq!(setups.len(), 2);
    assert!(setups.iter().any(|s| s.name == "Default 4-Osc"));
    assert!(setups.iter().any(|s| s.name == "Bridge Lead"));

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn save_and_load_roundtrip() {
    let setup = SynthSetup::from_json(DEFAULT_JSON).unwrap();
    let path = std::env::temp_dir().join(format!("modulus-setup-save-{}.json", std::process::id()));
    setup.save(&path).expect("save should work");
    let loaded = SynthSetup::load(&path).expect("load should work");
    assert_eq!(setup, loaded);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn setups_dir_is_a_path() {
    let dir: PathBuf = SynthSetup::setups_dir();
    assert!(dir.ends_with("Modulus/setups"));
}
