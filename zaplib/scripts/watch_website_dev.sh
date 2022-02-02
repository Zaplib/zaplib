#!/bin/bash

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}/../.."

cargo install cargo-watch

# For some reason mdbook touches image files on compilation, so we ignore those..
cargo watch --why --ignore 'docs/src/img/*' --watch zaplib/ --shell zaplib/scripts/build_website_dev.sh
