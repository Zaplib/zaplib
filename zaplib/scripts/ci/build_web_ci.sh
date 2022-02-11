#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

## Build
cd zaplib/web
yarn install
yarn build

## Test
yarn lint

# Build test suite as it is getting run in Jest
cargo zaplib build -p test_suite
yarn run jest --forceExit

## Publish
npm version 0.0.0-$(git rev-parse --short HEAD)
npm publish --tag canary
