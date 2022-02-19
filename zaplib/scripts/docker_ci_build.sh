#!/bin/bash

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}"

# Copy files that need to be transfered to the final image.
cp ../../rust-toolchain.toml .

# Warm the cache. Comment out this line and remove the local cache to build fresh.
docker pull janpaul123/zaplib-ci:latest

# Actually build, and tag with current commit hash.
TAG=$(git rev-parse HEAD | head -c 8)
docker build -f Dockerfile-ci --cache-from=janpaul123/zaplib-ci:latest -t zaplib-ci:$TAG .

# TODO(JP): In the future we might want to bust the cache periodically.
# See https://github.com/Zaplib/zaplib/issues/62

# Cleanup.
rm ./rust-toolchain.toml
