#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

# Build test_suite.wasm
cargo run -p cargo-zaplib -- build -p test_suite

# Build
pushd zaplib/web
    # Publish early, in case you want to use this even when there
    # are still some failing tests
    yarn install
    yarn run build
    export VERSION=0.0.0-$(git rev-parse --short HEAD)
    npm version $VERSION
    # Don't publish if this git hash has already been published.
    # This way we allow for rerunning the CI workspace.
    (npm view zaplib@$VERSION | grep tarball) || npm publish --tag canary

    # JS Tests
    yarn lint

    # Run jest (uses test suite)
    # --detectOpenHandles ensures that the tests hang if we leave any Web Workers open
    yarn run jest --detectOpenHandles
popd
