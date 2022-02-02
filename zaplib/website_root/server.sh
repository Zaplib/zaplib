#!/bin/bash

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}"

PORT=4848 ../zaplib/scripts/server.py
