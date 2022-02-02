#!/bin/bash

set -euo pipefail

# Check for Clippy errors
# Rules defined here represent intentional project wide configuration. To see outstanding Clippy TODOs, search
# the codebase for `Clippy TODO`.

# Some context for the below rule exceptions:
# single_match - this makes diffs a lot bigger, since adding or removing a match case will
#   require you to rewrite the whole expression
# too_many_arguments - it is very context dependent, and much harder to fix without larger
#   architecture refactoring
# comparison_chain - the proposed syntax of the Ordering enum is quite ugly
# branches_sharing_code - in certain contexts this is more readable
# many_single_char_names - we often pass in coordinates (x,y,z), which are meaningful
#   single character names
# manual_map - doesn't seem clearer in many cases

# cargo clippy --workspace --fix --allow-dirty --allow-staged --all-targets --
cargo clippy --workspace --all-targets -- \
    -D clippy::all \
    -A clippy::single_match \
    -A clippy::too_many_arguments \
    -A clippy::comparison_chain \
    -A clippy::branches_sharing_code \
    -A clippy::many_single_char_names \
    -A clippy::manual_map \
