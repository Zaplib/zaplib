#!/bin/bash

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}"

TAG=$(git rev-parse HEAD | head -c 8)

# Copy files that need to be transfered to the final image
cp ../../rust-toolchain.toml .

docker build -f Dockerfile-ci -t zaplib-ci:$TAG .

# Cleanup
rm ./rust-toolchain.toml

