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

* Build the website and watch for changes: `zaplib/scripts/watch_website_dev.sh` 
* Run local server on port 4848: `website_dev/server.sh` 
* Documentation sources are located at `zaplib/docs/src`



