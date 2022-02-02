#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

zaplib/scripts/ci/common.sh

export RUSTFLAGS="-D warnings"
cargo build --all-targets --workspace # builds everything in the workspace, including tests, etc
cargo run -p cargo-zaplib -- build --all-targets --workspace # same but for wasm
