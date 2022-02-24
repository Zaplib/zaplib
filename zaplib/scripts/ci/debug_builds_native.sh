#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

zaplib/scripts/ci/common.sh

export RUSTFLAGS="-D warnings"

# Builds everything in the workspace, including tests, etc.
# Note that it's important that this is a nightly build (which it currently is through
# rust-toolchain.toml), so we get notified about features that have stabilized, like
# https://github.com/rust-lang/rust/issues/58179#issuecomment-867793443
cargo build --all-targets --workspace
