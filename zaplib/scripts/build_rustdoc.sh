#!/bin/bash

set -euo pipefail

# --no-deps and individual package specification because otherwise some deps crash during rustdoc.
# BROWSER=echo and --open per https://github.com/rust-lang/cargo/issues/5562#issuecomment-887068135
# RUSTDOCFLAGS="-Dwarnings" so warnings are turned into errors.
RUSTDOCFLAGS="-Dwarnings" BROWSER=echo cargo doc --open --no-deps -p zaplib -p zaplib_components
