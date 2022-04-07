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

mkdir -p website_dev/zaplib/examples/example_bigedit/src/
cp zaplib/examples/example_bigedit/src/treeworld.rs website_dev/zaplib/examples/example_bigedit/src/treeworld.rs

cp zaplib/examples/example_flamegraph/index.html website_dev/example_flamegraph.html
cp zaplib/examples/example_lightning/index.html website_dev/example_lightning.html
cp zaplib/examples/example_charts/index.html website_dev/example_charts.html
cp zaplib/examples/example_shader/index.html website_dev/example_shader.html
cp zaplib/examples/example_single_button/index.html website_dev/example_single_button.html
cp zaplib/examples/example_lots_of_buttons/index.html website_dev/example_lots_of_buttons.html
cp zaplib/examples/example_bigedit/index.html website_dev/example_bigedit.html
cp zaplib/examples/example_image/index.html website_dev/example_image.html
cp zaplib/examples/example_text/index.html website_dev/example_text.html
cp zaplib/examples/test_bottom_bar/index.html website_dev/test_bottom_bar.html
cp zaplib/examples/test_geometry/index.html website_dev/test_geometry.html
cp zaplib/examples/test_layout/index.html website_dev/test_layout.html
cp zaplib/examples/test_many_quads/index.html website_dev/test_many_quads.html
cp zaplib/examples/test_multithread/index.html website_dev/test_multithread.html
cp zaplib/examples/test_padding/index.html website_dev/test_padding.html
cp zaplib/examples/test_popover/index.html website_dev/test_popover.html
cp zaplib/examples/test_shader_2d_primitives/index.html website_dev/test_shader_2d_primitives.html

CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_flamegraph --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_lightning --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_charts --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_shader --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_single_button --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_lots_of_buttons --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_bigedit --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_image --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p example_text --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p test_bottom_bar --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p test_geometry --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p test_layout --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p test_many_quads --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p test_multithread --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p test_padding --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p test_popover --release
CARGO_TARGET_DIR="website_dev/target" cargo run -p cargo-zaplib -- build -p test_shader_2d_primitives --release

pushd zaplib/web/
    yarn
    # TODO(JP): This takes quite long! Look into caching this step.
    yarn build
popd
mkdir -p website_dev/zaplib/web/dist/
cp -R zaplib/web/dist/* website_dev/zaplib/web/dist/

echo 'Website generated for development! Host using `cargo zaplib serve website_dev/ --port 4848`'
