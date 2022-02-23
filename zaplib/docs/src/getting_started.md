# Getting Started

## Installation

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Clone Zaplib:

```
git clone https://github.com/Zaplib/zaplib.git
```

3. Navigate to the repo, install the Cargo extension for Zaplib, and the dependencies:

```
cd zaplib
cargo install cargo-zaplib
cargo zaplib install-deps
```

If you're going to do local development of Zaplib, be sue to add the `--devel` flag which installs some more dependencies, like [CEF](./cef.md) binaries.

```
cargo zaplib install-deps --devel
```

## Examples

Now you're ready to run a simple example natively. Here are some fun ones to play with:
* `cargo run -p example_single_button`
* `cargo run -p example_charts`
* `cargo run -p example_text`
* `cargo run -p example_lightning --release` (best to do a release build; see below)
* `cargo run -p example_bigedit --release` (best to do a release build; see below)

Feel free to check out the `examples` directory for more examples to play with!

**Warning:** On Mac we currently have a memory leak bug, so some examples might crash after a while. Windows doens't work at all currently. Linux hasn't been tested very well recently. WebAssembly (below) should generally work well though. Early alpha software.. stay tuned!

##  WebAssembly Build

Of course, Zaplib is a WebAssembly framework, so let's run these in **Chrome** (more browser support coming soon):

1. Build all the examples:
   
```
cargo zaplib build --workspace
```

2. Run a server:

```
cargo zaplib serve
```

3. Navigate Chrome to: 

[`http://localhost:3000/zaplib/examples/example_single_button`](http://localhost:3000/zaplib/examples/example_single_button)

## Release Build

For a more performant build, add the `--release` flag, e.g.:


### Native

```
cargo run -p example_single_button --release
```

### WebAssembly

```
cargo zaplib build -p example_single_button --release
```

You'll also need to add the `?release` query param:

[`http://localhost:3000/zaplib/examples/example_single_button/?release`](http://localhost:3000/zaplib/examples/example_single_button/?release)


## Next Steps

1. Set up your [developer environment](./developer_environment.html).
2. Dive into some tutorials.
3. Look at some example code. [`example_single_button`](https://github.com/Zaplib/zaplib/blob/main/zaplib/examples/example_single_button/src/single_button.rs) is a good place to start if you're coming from ReactJS.
