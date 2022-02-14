<!---
TODO(Paras): Rustdoc for register_call_rust
Can we somehow bolded code changes between code snippets?
-->


# Tutorial: Integrating with JS

This guide will show you how to add Rust code to an existing JavaScript codebase, explaining how to:
* Create a WebAssembly instance and load your Rust code.
* Call functions in Rust and communicate results to JavaScript.

You can either follow this tutorial directly; creating the necessary files from scratch. It's easiest to create your code in the `zaplib` directory.

Or for a full working example, check out `tutorial_js_rust_bridge` in the `zaplib/examples` directory.

## Identifying a need
Let's say you have a JavaScript codebase which needs to calculate the sum of all values in an array.

```html
<!-- index.html -->
<html>
    <head>
        <script type="text/javascript" src="./index.js"></script>
    </head>
    <body>
        <div id="root"></div>
    </body>
</html>
```
```js
// index.js
const values = [1,2,3,4,5,6,7,8,9,10];
const sum = values.reduce((acc, v) => acc + v);

document.getElementById('root').textContent = sum;
```

This is a contrived example which does not need performance optimization, but importantly, one that locks the entire main thread while calculating results.

There are a few ways to make this better, in order:
* Moving to a promise-based approach with a loading state, so other interactions aren't blocked — this achieves *concurrency*.
* Moving this computation into a Web Worker — this achieves *parallelism* and better utilizes multi-core machines.
* Translating this computation to a *compiled language* (like Rust or C++) and attaching to a browser using WebAssembly — this lets us utilize the performance characteristics of other languages, which are usually better than JavaScript.

Zaplib provides a communication framework to do this last option with a bit more ease than other options today. Let's walk through how.

## Serving a WebAssembly binary
Let's start a new Zaplib application! We'll need to create a Rust entrypoint and add some boilerplating so it can compile. After this, we'll show how to actually execute this code.

Additionally, we'll add a basic version of our existing JavaScript logic as Rust code.
```rust,noplayground
// src/main.rs
use zaplib::*;

fn sum() {
    // Hardcode the values for now. Later, we'll show how to communicate parameters.
    let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let sum = values.iter().sum();

    // Log sum to console. Later, we'll actually return this to JavaScript.
    log!(sum);
}

fn call_rust(name: String, _params: Vec<ZapParam>) -> Vec<ZapParam> {
    if name == "sum" {
        sum();
    }

    vec![]
}

register_call_rust!(call_rust);
```

```toml
# Cargo.toml
[package]
name = "tutorial_js_rust_bridge"
version = "0.0.1"
edition = "2021"
publish = false

[dependencies]
zaplib = { path="../../main" }
```
<!--- TODO(Paras): What will be the path to zaplib in Cargo.toml? -->

### What's new?
We just added a lot, so here are the key things.
#### src/main.rs
This will be our Rust entrypoint for the package.
 - `use zaplib::*;` imports the Zaplib library.
 - `register_call_rust` lets us register Rust code as callable from JavaScript. The registered function will act on two arguments: a `name` field which specifies our input argument from JavaScript, and `params` with any input data. The function returns an output vector. We'll get to that in a bit.
<!--- TODO(Paras): Rustdoc for register_call_rust -->

#### Cargo.toml
This is our package manifest, needed when structuring any Rust application. For more information on basic Rust packaging, see [the official Cargo guide](https://doc.rust-lang.org/cargo/guide/creating-a-new-project.html). We specify `zaplib` in the dependencies.

### Compiling
Compile this into a WebAssembly binary by calling:
```
zaplib/scripts/build_website_dev.sh -p tutorial_js_rust_bridge
```
<!--- TODO(Paras): Not sure what the path will be for this script. -->

You'll now see a binary placed in `target/wasm32-unknown-unknown/debug/tutorial_js_rust_bridge.wasm`.

### Serving
To load this file on the Web, we'll need an HTTP server. There's no strict requirement on the backend, as long as:
 - The `wasm` file is served with the `application/wasm` MIME type.
 - CORS headers are set with at least:
 ```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
Access-Control-Allow-Origin: *
 ```
<!--- TODO(Paras): More restrictive CORS requirements probably exist. -->

If you already have the server running per instructions in [Getting Started](./getting_started.md), then great, you can keep using that! If you're interested in a more minimal server example, check out `zaplib/examples/tutorial_js_rust_bridge/server.py`.

## Connecting to web
Now that we have our backend ready, let's write our new JavaScript.

Our existing code is modified to be:
```js
// index.js
zaplib.initialize({
    wasmModule: `path/to/target/wasm32-unknown-unknown/debug/tutorial_js_rust_bridge.wasm`,
    defaultStyles: true
}).then(() => {
  zaplib.callRust('sum');
});
```

```html
<!-- index.html -->
<html>
    <head>
        <script type="text/javascript" src="/zaplib/web/dist/zaplib_runtime.js"></script>
        <script type="text/javascript" src="./index.js"></script>
    </head>
    <body>
        <div id="root"></div>
    </body>
</html>
```

### What's new?
- `zaplib.initialize`, with a path to the `.wasm` file. This assumes our web server is at the same port that served this HTML.
- `zaplib.callRust`, where the first parameter specifies a `name` of associated logic in Rust.
- Importing `zaplib_runtime` in our HTML.

### Results
Load up the web page — in the console you should see your summed up result. Hooray! Baby steps.

## Getting JavaScript inputs in Rust
This approach shows how to trigger Rust code from JavaScript, but is missing fundamentals, notably the ability to pass input or read output. Let's first start with inputs.

Here is our modified code.

```js
// index.js (after zaplib.initialize)
const values = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
zaplib.callRust('sum', [values]);
```

```rust,noplayground
// src/main.rs
use zaplib::*;

fn sum(values: &[u8]) {
    let sum = values.iter().sum();

    // Log sum to console. Later, we'll actually return this to JavaScript.
    log!(sum);
}

fn call_rust(name: String, params: Vec<ZapParam>) -> Vec<ZapParam> {
    if name == "sum" {
        let values = params[0].as_u8_slice();
        sum(&values);
    }

    vec![]
}

register_call_rust!(call_rust);
```

### What's new?
`callRust` can be passed a second parameter, a list of parameters of arbitrary length. Parameters must be either strings or [TypedArrays](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Typed_arrays). In the above case, we're using a `Uint8Array`.

Our callback in Rust must now read this value, casting the parameter to the correct type. For Uint8Arrays, we can use the `as_u8_slice()` convenience method for this. Now we can use this like any normal array!

## Getting Rust outputs into JavaScript
Outputs work with a similar parameter structure, with the ability to pass both strings and buffers.

Here is our modified code.
```js
// index.js (after zaplib.initialize)
const values = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
const [sumArray] = await zaplib.callRust('sum', [values]);
const sum = sumArray[0];
document.getElementById('root').textContent = sum;
```

```rust,noplayground
// src/main.rs
use zaplib::*;

fn sum(values: &[u8]) -> u8 {
    values.iter().sum()
}

fn call_rust(name: String, params: Vec<ZapParam>) -> Vec<ZapParam> {
    if name == "sum" {
        let values = params[0].as_u8_slice();
        let response = vec![sum(&values)].into_param();
        return vec![response];
    }

    vec![]
}

register_call_rust!(call_rust);
```

### What's new?
`callRust` will respond asynchronously with an array of parameters, so our function must now use async/await. We'll populate the first item of this array with our sum, which will be a buffer with one item.

In Rust, our function can now return a vector of results. Note that each result value must be of type `ZapParam` using the helper `into_param()`.

## Conclusion

We now have a web application which uses Zaplib to offload computations to Rust! To reiterate, this solution:
 - Has built-in parallelism, since Zaplib computations happen in Web Workers.
 - Offers Rust's trademark memory safety and performance.

This solution works well, but still has one big disadvantage regarding performance: copying data. In the above example, our provided Uint8Array will be copied every time this function is called into the WebAssembly memory.

To ensure great performance, we must instead structure our application to share memory across JavaScript and Rust, which we'll talk about in the [next tutorial](./tutorial_sharing_data.md).
