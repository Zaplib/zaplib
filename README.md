# ⚡ Zaplib

**Zaplib** is an open-source library for speeding up web applications using Rust and WebAssembly. It lets you write high-performance code in Rust, alongside your existing JavaScript code, using simple APIs.

# Development

## Installing

* [Install Rust](https://www.rust-lang.org/tools/install)
* Install the Cargo extension for Zaplib `cargo install cargo-zaplib`
* Run the dependency installation using Zaplib Cargo tool `cargo zaplib install-deps`

## Cargo extension

Zaplib provides a cargo extension that can be used to perform different tasks. The extension can be installed like this: `cargo install cargo-zaplib`

Optionally, you can use the tool directly like so `cargo run -p cargo-zaplib`, as any other project.

## Docs

* [Install Rust](https://www.rust-lang.org/tools/install)
* Run `zaplib/scripts/build_website_dev.sh` to generate our docs website. Follow the instructions on how to view them.
* When developing docs, use `zaplib/scripts/watch_website_dev.sh` to watch for changes.
* When publishing the docs, run `zaplib/scripts/build_website_prod.sh` which copies only the relevant files.

## License

Zaplib is distributed under the terms of both the MIT license and the Apache License (version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details. Third party license notices are available in [LICENSES-THIRD-PARTY](LICENSES-THIRD-PARTY).
