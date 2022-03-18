#!/bin/bash

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}/../.."

# For the development version we don't use a a fresh target directory every time since we want to
# make it cheap to run this script (e.g. for `watch_website_dev.sh`).

mkdir -p website_dev/

cp -R zaplib/website_root/* website_dev/

cargo install mdbook
mdbook build zaplib/docs --dest-dir ../../website_dev/docs/

# --no-deps and individual package specification because otherwise some deps crash during rustdoc.
# RUSTDOCFLAGS="-Dwarnings" so warnings are turned into errors.
CARGO_TARGET_DIR="website_dev/target" RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps -p zaplib -p zaplib_components

# Disabling (unused) wasm demos to temporarily fix Heroku timeouts (https://github.com/Zaplib/zaplib/issues/141)
# mkdir -p website_dev/zaplib/examples/example_bigedit/src/
# cp zaplib/examples/example_bigedit/src/treeworld.rs website_dev/zaplib/examples/example_bigedit/src/treeworld.rs
# cp zaplib/examples/example_bigedit/index.html website_dev/example_bigedit.html
# cp zaplib/examples/example_charts/index.html website_dev/example_charts.html
# cp zaplib/examples/example_lightning/index.html website_dev/example_lightning.html
# cp zaplib/examples/example_lots_of_buttons/index.html website_dev/example_lots_of_buttons.html
# cp zaplib/examples/example_shader/index.html website_dev/example_shader.html
# cp zaplib/examples/example_single_button/index.html website_dev/example_single_button.html
# cp zaplib/examples/example_text/index.html website_dev/example_text.html
# CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_bigedit --release
# CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_charts --release
# CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_lightning --release
# CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_lots_of_buttons --release
# CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_shader --release
# CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_single_button --release
# CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_text --release

pushd zaplib/web/
    yarn
    # TODO(JP): This takes quite long! Look into caching this step.
    yarn build
popd
mkdir -p website_dev/zaplib/web/dist/
cp -R zaplib/web/dist/* website_dev/zaplib/web/dist/

echo 'Website generated for development! Host using `cargo zaplib serve website_dev/ --port 4848`'
