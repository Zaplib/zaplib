#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

zaplib/scripts/ci/common.sh

# TODO(JP): Move to Dockerfile-ci?.
rustup component add rustfmt --toolchain nightly-2022-01-18-x86_64-unknown-linux-gnu

cargo fmt --all -- --check # checks formatting for all Rust files

zaplib/scripts/clippy.sh

# Make sure rustdoc works without warnings as well.
zaplib/scripts/build_rustdoc.sh
