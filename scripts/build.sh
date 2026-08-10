#!/usr/bin/env bash
# Build script for Modulus on macOS and Linux.
#
# Usage:
#   ./scripts/build.sh            # checks + release build + bundles
#   ./scripts/build.sh --skip-checks
#
# Output: target/bundled/*.vst3 and *.clap

set -euo pipefail

SKIP_CHECKS=0
if [[ "${1:-}" == "--skip-checks" ]]; then
    SKIP_CHECKS=1
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

if [[ "$SKIP_CHECKS" -eq 0 ]]; then
    echo "==> fmt"
    cargo fmt --all --check

    echo "==> clippy (-D warnings)"
    cargo clippy --workspace --all-targets -- -D warnings

    echo "==> building demo-module"
    cargo build -p demo-module

    echo "==> tests"
    if [[ "$(uname)" == "Darwin" ]]; then
        ext="dylib"
    else
        ext="so"
    fi
    export MODULUS_DEMO_MODULE="$root/target/debug/libdemo_module.$ext"
    cargo test --workspace
fi

echo "==> release build"
cargo build --release -p modulus-synth -p modulus-fx

echo "==> bundle VST3/CLAP"
cargo run -p xtask --release bundle

echo "Bundles:"
find "$root/target/bundled" -type f | sort