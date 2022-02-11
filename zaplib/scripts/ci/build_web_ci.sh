#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

# Build test suite as it is getting run in Jest
cargo zaplib build -p test_suite

cd zaplib/web
yarn install
yarn lint
yarn build
yarn run jest --forceExit
