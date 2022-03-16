#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

zaplib/scripts/ci/common.sh

export RUSTFLAGS="-D warnings"
cargo test --workspace # runs tests for the entire workspace
