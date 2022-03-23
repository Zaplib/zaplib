# Contributing

## Formatting

All Rust and JS/TS code must be formatted to pass our CI tests.

1. Format Rust:

```
cargo fmt
```

2. Format JS/TS:

```
yarn run prettier --write **/*.ts
```

We plan to add a Markdown formatter requirement.

## Tests

There are 2 types of tests available. The browser test suite is more extensive, but currently requires manual run. And the [Jest](https://jestjs.io/) tests that check the subset of Zaplib functionality using NodeJS environment.
### Browser tests

1. Build the test suite:

```
cargo zaplib build -p test_suite
```

2. Have local server running:

```
cargo zaplib serve
```

3. Navigate to <a href="http://localhost:3000/zaplib/web/test_suite/" target="_blank">http://localhost:3000/zaplib/web/test_suite/</a>

4. Click `Run All Tests`

### Browser tests ([Zapium](./zapium.md))

1. Install CEF (macOS Intel onl):

```
cargo zaplib install-deps --devel
``` 

2. Run the tests:

```
cargo run -p test_suite
``` 

3. Click `Run All Tests`

### Browser tests via chromedriver (similar to CI):

1. Install ChromeDriver:

```
brew install --cask chromedriver
```

2. Run chromedriver:

```
chromedriver
```

3. Build the examples:

```
cargo zaplib build --workspace
cargo zaplib build --workspace --release
```

4. Run the CI tests: 

```
cargo run -p zaplib_ci -- --webdriver-url http://localhost:9515
```

### Jest tests

1. Build Zaplib:

```
cd zaplib/web
yarn run watch 
```

2. Run the tests:

```
cd zaplib/web
yarn run jest --detectOpenHandles
```

Note: if Jest detects open handles, it may be due to unclosed Web Workers. Ensure that `zaplib.close()` is called after each test.

## Updating the documentation

The documentation files are located at [`zaplib/docs/src`](https://github.com/Zaplib/zaplib/tree/main/zaplib/docs/src). To server the docs locally:

1. Build the website and watch for changes: `zaplib/scripts/watch_website_dev.sh` 
2. Run local server: `cargo zaplib serve website_dev/ --port 4848` 

To update the prod website:

1. Run: `zaplib/scripts/build_website_prod.sh`
2. Clone zaplib-site locally: `git clone https://github.com/janpaul123/zaplib-site.git`
3. Copy the built website files file to zaplib-site: `cp -r website/* ../zaplib-site/`


## Building the NPM Package

1. Install Node & NPM & yarn
2. Build the package:

```
cd zaplib/web && yarn && yarn watch
```
