# Getting Started

First let's install some dependencies:
* [Install Rust](https://www.rust-lang.org/tools/install)
* Clone the repo: `git clone https://github.com/Zaplib/zaplib.git`
* Navigate to the repo: `cd zaplib`
* Install the Cargo extension for Zaplib `cargo install cargo-zaplib`
* Run the dependency installation using Zaplib Cargo tool `cargo zaplib install-deps`
  * If you're going to do local development of Zaplib, be sue to add the `--devel` flag which installs some more dependencies, like [CEF](https://github.com/chromiumembedded) binaries.

Now you're ready to run a simple example natively. Here are some fun ones to play with:
* `cargo run -p example_single_button`
* `cargo run -p example_charts`
* `cargo run -p example_text`
* `cargo run -p example_lightning` (heavy; best to do a release build; see below)
* `cargo run -p example_bigedit` (heavy; best to do a release build; see below)

**Warning:** On Mac we currently have a memory leak bug, so some examples might crash after a while. Windows doens't work at all currently. Linux hasn't been tested very well recently. WebAssembly (below) should generally work well though. Early alpha software.. stay tuned!

For a more performant build, add the `--release` flag, e.g.:
* `cargo run -p example_single_button --release`

Of course, Zaplib is primarily a framework for WebAssembly, so let's run these examples in a browser:
* Download the latest version of a modern browser, like [Chrome](https://www.google.com/chrome/).
* In a separate terminal window, run a basic server: `zaplib/scripts/server.py` (Note that this still requires Python 2).
* In another separate terminal window, start yarn to build the Zaplib javascript files:
  * `cd zaplib/web && yarn && yarn watch`
* Build one of the examples using the Zaplib Cargo tool, e.g.:
  * `cargo zaplib build -p example_single_button`
* Navigate your browser to:
  * [`http://localhost:3000/zaplib/examples/example_single_button`](http://localhost:3000/zaplib/examples/example_single_button)
* Again, for a more performant build, add the `--release` flag, e.g.:
  * `cargo zaplib build -p example_single_button --release`
* With a release build, add a `?release` flag to the URL:
  * [`http://localhost:3000/zaplib/examples/example_single_button/?release`](http://localhost:3000/zaplib/examples/example_single_button/?release)

Feel free to check out the `examples` directory for more examples to play with!

To view automatically generated API documentation, run:
* `zaplib/scripts/build_rustdoc.sh`

If you're wondering what to do next, here are some options:
* Set up your [tooling](./basic_tooling.md).
* Dive into some tutorials.
* Look at the code for one of the examples (`example_single_button` is a great simple one to start with) and try to modify it.
