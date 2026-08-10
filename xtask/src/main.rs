use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PLUGINS: &[(&str, &str)] = &[
    ("modulus-synth", "Modulus"),
    ("modulus-fx", "Modulus FX"),
];

/// Per-host-OS artifact layout (file name suffix, VST3 Contents subdir).
fn host_os_layout() -> (&'static str, &'static str) {
    match env::consts::OS {
        "windows" => ("x86_64-win", "dll"),
        "macos" => ("macOS", "dylib"),
        "linux" => ("x86_64-linux", "so"),
        other => {
            eprintln!("unsupported host OS for bundling: {other}");
            std::process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let subcommand = args.get(1).map(String::as_str).unwrap_or("bundle");
    if subcommand != "bundle" {
        eprintln!("usage: cargo run -p xtask --release [bundle] [plugin-name-filter]");
        std::process::exit(1);
    }
    let filter = args.get(2).map(String::as_str);

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let target_dir = root.join("target");
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    for &(package, plugin_name) in PLUGINS {
        if let Some(filter) = filter {
            let matches = package.contains(filter) || plugin_name.to_lowercase().contains(filter);
            if !matches {
                continue;
            }
        }

        println!("Building `{package}` (release)...");
        let status = Command::new(&cargo)
            .args(["build", "--release", "--package", package])
            .current_dir(root)
            .status()
            .expect("failed to spawn cargo");
        if !status.success() {
            eprintln!("build failed for `{package}`");
            std::process::exit(1);
        }

        let lib_name = package.replace('-', "_");
        let (os_dir, ext) = host_os_layout();
        let artifact = release_artifact(&target_dir, &lib_name, ext);
        if !artifact.exists() {
            eprintln!("expected build artifact not found: {}", artifact.display());
            std::process::exit(1);
        }

        let vst3_file = target_dir
            .join("bundled")
            .join(format!("{plugin_name}.vst3"))
            .join("Contents")
            .join(os_dir)
            .join(format!("{plugin_name}.vst3"));
        let clap_file = target_dir
            .join("bundled")
            .join(format!("{plugin_name}.clap"))
            .join(format!("{plugin_name}.clap"));

        for bundle_file in [&vst3_file, &clap_file] {
            if let Some(parent) = bundle_file.parent() {
                fs::create_dir_all(parent).expect("failed to create bundle directory");
            }
            fs::copy(&artifact, bundle_file).expect("failed to copy bundle artifact");
            make_executable(bundle_file);
            println!("Bundled -> {}", bundle_file.display());
        }
    }
}

/// The release artifact produced by cargo for the given library name.
fn release_artifact(target_dir: &Path, lib_name: &str, ext: &str) -> PathBuf {
    match ext {
        "dll" => target_dir.join("release").join(format!("{lib_name}.dll")),
        "dylib" => target_dir.join("release").join(format!("lib{lib_name}.dylib")),
        "so" => target_dir.join("release").join(format!("lib{lib_name}.so")),
        _ => unreachable!(),
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = match fs::metadata(path) {
        Ok(m) => m.permissions(),
        Err(err) => {
            eprintln!("failed to stat {}: {err}", path.display());
            std::process::exit(1);
        }
    };
    perms.set_mode(perms.mode() | 0o755);
    let _ = fs::set_permissions(path, perms);
}

#[cfg(windows)]
fn make_executable(_path: &Path) {
    // Windows executability is implied by the file extension.
}