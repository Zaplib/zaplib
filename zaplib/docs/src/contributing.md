# Contributing

## Tests

There are 2 types of tests available. The browser test suite is more extensive, but currently requires manual run. And the [Jest](https://jestjs.io/) tests that check the subset of Zaplib functionality using NodeJS environment.

* Running browser tests
  * Build the test suite: `cargo zaplib build -p test_suite`
  * Have local server running: `zaplib/scripts/server.py`
  * Navigate to `http://localhost:3000/zaplib/test_suite/` and click `Run All Tests` button
* Running jest tests
  * `cd zaplib/web && yarn run jest`

## Updating the documentation

The documentation files are located at `zaplib/docs/src`. To server the docs locally:

1. Build the website and watch for changes: `zaplib/scripts/watch_website_dev.sh` 
2. Run local server on port 4848: `website_dev/server.sh` 

To update the prod website:

1. Run: `zaplib/scripts/build_website_prod.sh`
2. Clone zaplib-site locally: `git clone https://github.com/janpaul123/zaplib-site.git`
3. Copy the built website files file to zaplib-site: `cp -r website/* ../zaplib-site/`
