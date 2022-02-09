#!/bin/bash

set -euo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}"

TAG=$(git rev-parse HEAD | head -c 8)

docker tag zaplib-ci:$TAG janpaul123/zaplib-ci:latest
docker push janpaul123/zaplib-ci:latest
