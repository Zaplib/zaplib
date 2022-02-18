#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

zaplib/scripts/ci/common.sh

export RUSTFLAGS="-D warnings"

# Run a check (not a build) for the various target triples.
cargo check --all-targets --workspace --target x86_64-unknown-linux-gnu --exclude tutorial_js_rust_bridge
cargo check --all-targets --workspace --target wasm32-unknown-unknown
# `--no-default-features` is to disable TLS since it breaks cross-compilation
# `--exclude zaplib_cef(_sys)` and `test_suite` since we currently don't support cross-compiling with CEF.
# `--exclude cargo-zaplib` because of an openssl dependency that doesn't support cross-compiling.
# `--exclude zaplib_ci` because of a `std-sys` dependency that doesn't support cross-compiling.
cargo check --all-targets --workspace --target x86_64-apple-darwin --no-default-features --exclude zaplib_cef --exclude zaplib_cef_sys --exclude test_suite --exclude tutorial_js_rust_bridge --exclude cargo-zaplib --exclude zaplib_ci
cargo check --all-targets --workspace --target x86_64-pc-windows-msvc --no-default-features --exclude zaplib_cef --exclude zaplib_cef_sys --exclude test_suite --exclude tutorial_js_rust_bridge --exclude cargo-zaplib --exclude zaplib_ci
cargo check --all-targets --workspace --target x86_64-pc-windows-gnu --no-default-features --exclude zaplib_cef --exclude zaplib_cef_sys --exclude test_suite --exclude tutorial_js_rust_bridge --exclude cargo-zaplib --exclude zaplib_ci
