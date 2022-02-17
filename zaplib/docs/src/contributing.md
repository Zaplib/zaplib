# Contributing

## Tests

There are 2 types of tests available. The browser test suite is more extensive, but currently requires manual run. And the [Jest](https://jestjs.io/) tests that check the subset of Zaplib functionality using NodeJS environment.

* Running browser tests
  * Build the test suite: `cargo zaplib build -p test_suite`
  * Have local server running: `cargo zaplib serve`
  * Navigate to `http://localhost:3000/zaplib/web/test_suite/` and click `Run All Tests`
  * Test CEF by running `cargo run -p test_suite` and clicking `Run All Tests` (macOS Intel only, and first install CEF using `cargo zaplib install-deps --devel`).
* Running jest tests
  * `cd zaplib/web && yarn run jest`

## Updating the documentation

The documentation files are located at [`zaplib/docs/src`](https://github.com/Zaplib/zaplib/tree/main/zaplib/docs/src). To server the docs locally:

1. Build the website and watch for changes: `zaplib/scripts/watch_website_dev.sh` 
2. Run local server: `cargo zaplib serve website_dev/ --port 4848` 

To update the prod website:

1. Run: `zaplib/scripts/build_website_prod.sh`
2. Clone zaplib-site locally: `git clone https://github.com/janpaul123/zaplib-site.git`
3. Copy the built website files file to zaplib-site: `cp -r website/* ../zaplib-site/`
