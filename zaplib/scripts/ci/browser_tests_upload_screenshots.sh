#!/bin/bash

set -euxo pipefail

# Per https://stackoverflow.com/a/16349776; go to repo root
cd "${0%/*}/../../.."

# Screenshots are saved in `screenshots/`. Previous ones in `previous_screenshots/`. Let's compare!
# `--ignoreChange` makes it so this call doesn't fail when there are changed screenshots; we don't
# want to block merging in that case.
# We use `--matchingThreshold` and `--thresholdPixel` because the GPU-rendered images come out slightly
# different now and then. We suspect that this is because of differences in GPU hardware between
# Browserstack runs.
zaplib/web/node_modules/.bin/reg-cli screenshots/ previous_screenshots/ diff_screenshots/ --report ./index.html --json ./reg.json --ignoreChange --enableAntialias --matchingThreshold 0.15 --thresholdPixel 10
# Now let's bundle everything up in screenshots_report/
mkdir screenshots_report/
mv index.html screenshots_report/
mv reg.json screenshots_report/
mv screenshots/ screenshots_report/
mv previous_screenshots/ screenshots_report/
mv diff_screenshots/ screenshots_report/
aws s3 cp --recursive screenshots_report/ s3://zaplib-screenshots/$GITHUB_SHA

if grep --fixed-strings '"newItems":[]' screenshots_report/reg.json && grep --fixed-strings '"deletedItems":[]' screenshots_report/reg.json && grep --fixed-strings '"failedItems":[]' screenshots_report/reg.json
then
  echo "SCREENSHOT_GITHUB_MESSAGE=[✅ No screenshot diffs found.](http://zaplib-screenshots.s3-website-us-east-1.amazonaws.com/$GITHUB_SHA)" >> $GITHUB_ENV
else
  if grep --fixed-strings '"deletedItems":[]' screenshots_report/reg.json && grep --fixed-strings '"failedItems":[]' screenshots_report/reg.json && grep --fixed-strings '"passedItems":[]' screenshots_report/reg.json
  then
    echo "SCREENSHOT_GITHUB_MESSAGE=[⚠️ Only new screenshots found.](http://zaplib-screenshots.s3-website-us-east-1.amazonaws.com/$GITHUB_SHA) This typically happens when the base commit screenshots were not built yet; just rerun the workspace. If that doesn't help, please contact a maintainer for help." >> $GITHUB_ENV
  else
    echo "SCREENSHOT_GITHUB_MESSAGE=[🤔 Screenshot diffs found.](http://zaplib-screenshots.s3-website-us-east-1.amazonaws.com/$GITHUB_SHA) Please look at the screenshots and tag this comment with 👍 or 👎. Only merge when both the PR author and a reviewer are happy with the changes." >> $GITHUB_ENV
  fi
fi

