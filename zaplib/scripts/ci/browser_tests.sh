#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

zaplib/scripts/build_website_dev.sh

# Build the wasm packages that we need in tests.
# We specify these builds individually in order to save on build times,
# since this job is already pretty slow now.
cargo run -p cargo-zaplib -- build -p test_suite
cargo run -p cargo-zaplib -- build -p tutorial_2d_rendering_step1
cargo run -p cargo-zaplib -- build -p tutorial_2d_rendering_step2
cargo run -p cargo-zaplib -- build -p tutorial_2d_rendering_step3
cargo run -p cargo-zaplib -- build -p tutorial_3d_rendering_step2
cargo run -p cargo-zaplib -- build -p tutorial_3d_rendering_step3
cargo run -p cargo-zaplib -- build -p tutorial_hello_thread
cargo run -p cargo-zaplib -- build -p tutorial_hello_world_canvas
cargo run -p cargo-zaplib -- build -p tutorial_hello_world_console
cargo run -p cargo-zaplib -- build -p tutorial_js_rust_bridge
cargo run -p cargo-zaplib -- build -p tutorial_ui_components
cargo run -p cargo-zaplib -- build -p tutorial_ui_layout
cargo run -p cargo-zaplib -- build --release -p example_charts
cargo run -p cargo-zaplib -- build --release -p example_image
cargo run -p cargo-zaplib -- build --release -p example_flamegraph
cargo run -p cargo-zaplib -- build --release -p example_lots_of_buttons
cargo run -p cargo-zaplib -- build --release -p example_single_button
cargo run -p cargo-zaplib -- build --release -p example_text
cargo run -p cargo-zaplib -- build --release -p test_bottom_bar
cargo run -p cargo-zaplib -- build --release -p test_geometry
cargo run -p cargo-zaplib -- build --release -p test_layout
cargo run -p cargo-zaplib -- build --release -p test_padding
cargo run -p cargo-zaplib -- build --release -p test_popover

# Integration tests with Browserstack (uses test suite)
# Local identifier is necessary to be able to run multiple jobs in parallel.
export BROWSERSTACK_LOCAL_IDENTIFIER=$(echo $RANDOM$RANDOM$RANDOM)
BrowserStackLocal --key $BROWSERSTACK_KEY --debug-utility --daemon start --local-identifier $BROWSERSTACK_LOCAL_IDENTIFIER
cargo run -p zaplib_ci -- --webdriver-url "https://jpposma_0ZuiXP:${BROWSERSTACK_KEY}@hub-cloud.browserstack.com/wd/hub" --browserstack-local-identifier $BROWSERSTACK_LOCAL_IDENTIFIER
