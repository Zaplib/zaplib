#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776
cd "${0%/*}"

cd ../../web
yarn install
yarn lint
yarn build
yarn run jest --forceExit
