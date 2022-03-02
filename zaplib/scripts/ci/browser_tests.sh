#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

# Build the wasm packages that we need in tests.
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
cargo run -p cargo-zaplib -- build --release -p example_lots_of_buttons
cargo run -p cargo-zaplib -- build --release -p example_shader
cargo run -p cargo-zaplib -- build --release -p example_single_button
cargo run -p cargo-zaplib -- build --release -p example_text
cargo run -p cargo-zaplib -- build --release -p test_bottom_bar
cargo run -p cargo-zaplib -- build --release -p test_geometry
cargo run -p cargo-zaplib -- build --release -p test_layout
cargo run -p cargo-zaplib -- build --release -p test_padding
cargo run -p cargo-zaplib -- build --release -p test_popover

# Build
pushd zaplib/web
    # Dev build (instead of prod, so we get better stack traces)
    yarn
    yarn run build
popd

# Integration tests with Browserstack (uses test suite)
# Local identifier is necessary to be able to run multiple jobs in parallel.
export BROWSERSTACK_LOCAL_IDENTIFIER=$(echo $RANDOM$RANDOM$RANDOM)
BrowserStackLocal --key $BROWSERSTACK_KEY --debug-utility --daemon start --local-identifier $BROWSERSTACK_LOCAL_IDENTIFIER
cargo run -p zaplib_ci -- --webdriver-url "https://jpposma_0ZuiXP:${BROWSERSTACK_KEY}@hub-cloud.browserstack.com/wd/hub" --browserstack-local-identifier $BROWSERSTACK_LOCAL_IDENTIFIER

# Screenshots are saved in `screenshots/`. Previous ones in `previous_screenshots/`. Let's compare!
# `--ignoreChange` makes it so this call doesn't fail when there are changed screenshots; we don't
# want to block merging in that case.
zaplib/web/node_modules/.bin/reg-cli screenshots/ previous_screenshots/ diff_screenshots/ --report ./index.html --json ./reg.json --ignoreChange --enableAntialias --matchingThreshold 0.05
# Now let's bundle everything up in screenshots_report/
mkdir screenshots_report/
mv index.html screenshots_report/
mv reg.json screenshots_report/
mv screenshots/ screenshots_report/
mv previous_screenshots/ screenshots_report/
mv diff_screenshots/ screenshots_report/
aws s3 cp --recursive screenshots_report/ s3://zaplib-screenshots/$GITHUB_SHA

if grep --fixed-strings '"newItems":[]' screenshots_report/reg.json && grep --fixed-strings '"deletedItems":[]' screenshots_report/reg.json && grep --fixed-strings '"failedItems":[]' screenshots_report/reg.json
then
  echo "SCREENSHOT_GITHUB_MESSAGE=[✅ No screenshot diffs found.](http://zaplib-screenshots.s3-website-us-east-1.amazonaws.com/$GITHUB_SHA)" >> $GITHUB_ENV
else
  if grep --fixed-strings '"deletedItems":[]' screenshots_report/reg.json && grep --fixed-strings '"failedItems":[]' screenshots_report/reg.json && grep --fixed-strings '"passedItems":[]' screenshots_report/reg.json
  then
    echo "SCREENSHOT_GITHUB_MESSAGE=[⚠️ Only new screenshots found.](http://zaplib-screenshots.s3-website-us-east-1.amazonaws.com/$GITHUB_SHA) This typically happens when the base commit screenshots were not built yet; just rerun the workspace. If that doesn't help, please contact a maintainer for help." >> $GITHUB_ENV
  else
    echo "SCREENSHOT_GITHUB_MESSAGE=[🤔 Screenshot diffs found.](http://zaplib-screenshots.s3-website-us-east-1.amazonaws.com/$GITHUB_SHA) Please look at the screenshots and tag this comment with 👍 or 👎. Only merge when both the PR author and a reviewer are happy with the changes." >> $GITHUB_ENV
  fi
fi

