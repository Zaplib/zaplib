#!/bin/bash

set -euxo pipefail

# If we're on main already, compare against our parent commit.
# Otherwise, compare against the merge-base commit on main.
#
# Export the environment variable using
# https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-an-environment-variable
#
# Based on https://github.com/cruise-automation/webviz/blob/6827f186a/.circleci/find-screenshots-compare-commit.sh
if [ $(git rev-parse HEAD) = $(git merge-base origin/main HEAD) ]
then
  echo "SCREENSHOT_COMPARE_COMMIT_HASH=$(git rev-parse HEAD~1)" >> $GITHUB_ENV
else
  echo "SCREENSHOT_COMPARE_COMMIT_HASH=$(git merge-base origin/main HEAD)" >> $GITHUB_ENV
fi
