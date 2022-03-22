#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

zaplib/scripts/ci/common.sh

export RUSTFLAGS="-D warnings"
# Note that we don't add `--all-targets` here, because (for some reason)
# that causes tests not to run at all!
cargo test --workspace # runs tests for the entire workspace
