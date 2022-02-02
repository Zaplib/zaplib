#!/bin/bash

set -euo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

# TODO(JP): The path where we put CEF originally is a bit funky, so it would be nice to clean
# that up at some point. Still, this is better than downloading it from the internet again..
cp -r /tmp/cef_binary_* zaplib/main/bind/cef-sys/deps/
