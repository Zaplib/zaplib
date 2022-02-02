#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

zaplib/scripts/ci/common.sh

export RUSTFLAGS="-D warnings"
cargo build --release # builds a standard release build for the current operating system
cargo run -p cargo-zaplib -- build --release # release build for wasm only
