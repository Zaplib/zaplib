#!/bin/bash

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}/../.."

# We build to a fresh target directory to make sure we don't have any stale files that we
# copy to the website.
rm -rf website_dev/

# Run the developer version of this script.
zaplib/scripts/build_website_dev.sh

# Copy over just the files that we actually want to ship.
rm -rf website/
mkdir website/
cp -R zaplib/website_root/* website/
cp -R website_dev/docs website/docs
mkdir website/target
cp -R website_dev/target/doc website/target/doc
cp -R website_dev/*.html website/
mkdir -p website/target/wasm32-unknown-unknown/release/
cp website_dev/target/wasm32-unknown-unknown/release/*.wasm website/target/wasm32-unknown-unknown/release/
mkdir -p website/zaplib/web/dist/
cp website_dev/zaplib/web/dist/* website/zaplib/web/dist/
mkdir -p website/zaplib/examples/example_bigedit/src/
cp website_dev/zaplib/examples/example_bigedit/src/treeworld.rs website/zaplib/examples/example_bigedit/src/treeworld.rs

echo 'Website generated for publishing! Host using `zaplib serve website/ --port 4848` or publish `website/`'
