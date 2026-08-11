//! Offline patch player: renders a Lua patch to a WAV file.
//!
//! Usage:
//!
//! ```text
//! cargo run -p modulus-core --example patch_player
//! MODULUS_PATCH=scripts/lua/example_patch.lua \
//!   MODULUS_OUTPUT=target/patch_output.wav \
//!   cargo run -p modulus-core --example patch_player
//! ```
//!
//! The default renders two seconds of `scripts/lua/example_patch.lua`.

use std::path::Path;

use modulus_core::modules::{builtin_registry, lua, ModuleEvents};

const SAMPLE_RATE: f32 = 44_100.0;
const DURATION_SECS: f32 = 2.0;

fn main() {
    let patch_path = std::env::var("MODULUS_PATCH")
        .unwrap_or_else(|_| "scripts/lua/example_patch.lua".to_string());
    let output_path =
        std::env::var("MODULUS_OUTPUT").unwrap_or_else(|_| "target/patch_output.wav".to_string());

    let source = std::fs::read_to_string(&patch_path)
        .unwrap_or_else(|err| panic!("failed to read patch {patch_path}: {err}"));

    let registry = builtin_registry();
    let mut graph = lua::build_patch(&registry, &source).expect("patch should compile");
    graph.prepare(SAMPLE_RATE);

    let sample_count = (SAMPLE_RATE * DURATION_SECS) as usize;
    let mut samples: Vec<i16> = Vec::with_capacity(sample_count * 2);

    let mut frame = [0.0_f32; 2];
    let mut peak = 0.0_f32;
    let mut events = ModuleEvents {
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
        "modules: {:?}  peak: {peak:.4}  silence: {}",
        graph.module_names(),
        peak < 0.001
    );
    if peak < 0.001 {
        eprintln!("warning: rendered output is silent");
        std::process::exit(1);
    }
    println!("wrote {output_path} ({} samples)", samples.len() / 2);
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
