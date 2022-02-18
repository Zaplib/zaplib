#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

# Build
pushd zaplib/web
    yarn install
    yarn build

    # Publish early, in case you want to use this even when there
    # are still some failing tests
    npm version 0.0.0-$(git rev-parse --short HEAD)
    npm publish --tag canary

    # JS Tests
    yarn lint

    # Build test suite
    cargo run -p cargo-zaplib -- build -p test_suite

    # Run jest (uses test suite)
    yarn run jest --forceExit
popd

# Integration tests with Browserstack (uses test suite)
# Local identifier is necessary to be able to run multiple jobs in parallel.
export BROWSERSTACK_LOCAL_IDENTIFIER=$(echo $RANDOM$RANDOM$RANDOM)
BrowserStackLocal --key $BROWSERSTACK_KEY --debug-utility --daemon start --local-identifier $BROWSERSTACK_LOCAL_IDENTIFIER
cargo run -p zaplib_ci -- --webdriver-url "https://jpposma_0ZuiXP:${BROWSERSTACK_KEY}@hub-cloud.browserstack.com/wd/hub" --browserstack-local-identifier $BROWSERSTACK_LOCAL_IDENTIFIER
