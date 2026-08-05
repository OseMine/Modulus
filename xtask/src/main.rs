use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const PLUGINS: &[(&str, &str)] = &[
    ("modulus-synth", "Modulus"),
    ("modulus-fx", "Modulus FX"),
];

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
    let release_dir = target_dir.join("release");
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

        let dll = release_dir.join(format!("{}.dll", package.replace('-', "_")));
        if !dll.exists() {
            eprintln!("expected build artifact not found: {}", dll.display());
            std::process::exit(1);
        }

        let vst3_file = target_dir
            .join("bundled")
            .join(format!("{plugin_name}.vst3"))
            .join("Contents")
            .join("x86_64-win")
            .join(format!("{plugin_name}.vst3"));
        let clap_file = target_dir
            .join("bundled")
            .join(format!("{plugin_name}.clap"))
            .join(format!("{plugin_name}.clap"));

        for bundle_file in [&vst3_file, &clap_file] {
            if let Some(parent) = bundle_file.parent() {
                fs::create_dir_all(parent).expect("failed to create bundle directory");
            }
            fs::copy(&dll, bundle_file).expect("failed to copy bundle artifact");
            println!("Bundled -> {}", bundle_file.display());
        }
    }
}
