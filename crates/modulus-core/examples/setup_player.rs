//! Offline setup player: renders a synth setup config to a WAV file.
//!
//! Usage:
//!
//! ```text
//! cargo run -p modulus-core --example setup_player
//! MODULUS_SETUP=setups/bridge_lead.json \
//!   MODULUS_OUTPUT=target/setup_output.wav \
//!   cargo run -p modulus-core --example setup_player
//! ```
//!
//! The default renders two seconds of `setups/default.json` (the default
//! 4-oscillator configuration).
//!
//! Setups without a `MODULUS_SETUP` env var: both `setups/default.json`
//! and the per-user `Modulus/setups` directory are considered.

use std::path::{Path, PathBuf};

use modulus_core::modules::{builtin_registry, ModuleEvents};
use modulus_core::synth_setup::SynthSetup;

const SAMPLE_RATE: f32 = 44_100.0;
const DURATION_SECS: f32 = 2.0;

fn main() {
    let (setup_paths, description) = if let Ok(path) = std::env::var("MODULUS_SETUP") {
        (vec![PathBuf::from(path)], "MODULUS_SETUP".to_string())
    } else {
        let mut paths = SynthSetup::scan_dir(&SynthSetup::setups_dir());
        paths.insert(0, PathBuf::from("setups/default.json"));
        (paths, "default dir + per-user dir".to_string())
    };

    let output_path =
        std::env::var("MODULUS_OUTPUT").unwrap_or_else(|_| "target/setup_output.wav".to_string());

    let registry = builtin_registry();
    let mut rendered_any = false;
    for setup_path in &setup_paths {
        let Ok(setup) = SynthSetup::load(setup_path) else {
            continue;
        };
        println!(
            "rendering setup '{}' from {} (found via {description})",
            setup.name,
            setup_path.display()
        );

        let mut graph = setup.build(&registry).expect("setup should build");
        graph.prepare(SAMPLE_RATE);
        let sample_count = (SAMPLE_RATE * DURATION_SECS) as usize;
        let mut samples: Vec<i16> = Vec::with_capacity(sample_count * 2);

        let mut frame = [0.0_f32; 2];
        let mut peak = 0.0_f32;
        let mut events = ModuleEvents {
            note_on: None,
            note_off: None,
            time_secs: 0.0,
            tuning_hz: 440.0,
        };

        // Simple pattern: note 60 for 1s, then note 67 (perfect fifth) for 1s.
        for index in 0..sample_count {
            let t = index as f32 / SAMPLE_RATE;
            events.time_secs = t as f64;
            if index == 0 {
                graph.note_on(60, 1.0, 440.0);
            }
            if index == sample_count / 2 {
                graph.note_off(60);
                graph.note_on(67, 1.0, 440.0);
            }

            graph.process_frame(&mut frame, &events, SAMPLE_RATE);
            peak = peak.max(frame[0].abs()).max(frame[1].abs());
            samples.push((frame[0].clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
            samples.push((frame[1].clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
            frame = [0.0; 2];
        }

        write_wav(Path::new(&output_path), SAMPLE_RATE as u32, &samples);
        println!(
            "soundgens: {}  peak: {peak:.4}  silence: {}",
            graph.soundgen_count(),
            peak < 0.001
        );
        if peak < 0.001 {
            eprintln!("warning: rendered output is silent");
            std::process::exit(1);
        }
        println!("wrote {output_path} ({} samples)", samples.len() / 2);
        rendered_any = true;
        break;
    }

    if !rendered_any {
        eprintln!("no setup config found (tried {description})");
        eprintln!(
            "hint: write setups to {} or set MODULUS_SETUP",
            SynthSetup::setups_dir().display()
        );
        std::process::exit(1);
    }
}

/// Minimal 16-bit stereo PCM WAV writer.
fn write_wav(path: &Path, sample_rate: u32, samples: &[i16]) {
    use std::io::Write;

    let bytes_per_sample = 2u16;
    let channel_count = 2u16;
    let byte_rate = sample_rate * bytes_per_sample as u32 * channel_count as u32;
    let data_len = samples.len() as u32 * bytes_per_sample as u32;

    let file = std::fs::File::create(path).expect("failed to create output file");
    let mut writer = std::io::BufWriter::new(file);
    writer.write_all(b"RIFF").unwrap();
    writer.write_all(&(36 + data_len).to_le_bytes()).unwrap();
    writer.write_all(b"WAVE").unwrap();
    writer.write_all(b"fmt ").unwrap();
    writer.write_all(&16u32.to_le_bytes()).unwrap();
    writer.write_all(&1u16.to_le_bytes()).unwrap();
    writer.write_all(&channel_count.to_le_bytes()).unwrap();
    writer.write_all(&sample_rate.to_le_bytes()).unwrap();
    writer.write_all(&byte_rate.to_le_bytes()).unwrap();
    writer
        .write_all(&(bytes_per_sample * channel_count).to_le_bytes())
        .unwrap();
    writer.write_all(&bytes_per_sample.to_le_bytes()).unwrap();
    writer.write_all(b"data").unwrap();
    writer.write_all(&data_len.to_le_bytes()).unwrap();
    for sample in samples {
        writer.write_all(&sample.to_le_bytes()).unwrap();
    }
}
