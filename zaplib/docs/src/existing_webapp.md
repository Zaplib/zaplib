# Integrating with existing webapps

Zaplib is still very experimental. Here are some things to keep in mind when integrating Zaplib in an existing application.
1. We recommend saying hi in our [Slack](/slack.html), so we can work with you on the integration.
2. Thoroughly read the docs, especially about the "JS-Rust bridge".
3. There are some further notes below specific to integration.

## Versioning

We don't release proper versions yet. Instead, you should pick a git commit and pin to that. In your `Cargo.toml`:

```toml
[dependencies]
zaplib = { git = "https://github.com/Zaplib/zaplib", rev="c015a1e" }
```

And in `package.json` (a version like this is pushed automatically to [npm](https://www.npmjs.com/) on every commit):

```js
"dependencies": {
    "zaplib": "0.0.0-c015a1e"
}
```

When upgrading, be sure to update both `Cargo.toml` and `package.json`, and be sure to follow along in [Slack](/slack.html) to learn about API changes.

## Jest integration

Zaplib can run in the [Jest testing framework](https://jestjs.io/). Following [Tutorial: Integrating with JS](./tutorial_js_rust_bridge.md), let's assume we have a `sum` hook for `callRust` defined:

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

Create a `jest.config.js` file with the following settings:

```js
/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
// eslint-disable-next-line no-undef
module.exports = {
  testEnvironment: "jest-environment-jsdom",
};
```

Then the simple Jest test would look like this:

```js
// example-jest.test.js

// Import set of polyfills to run zaplib in NodeJS environment
require("zaplib/dist/zaplib_nodejs_polyfill");

const zaplib = require("zaplib");
const fs = require("fs");

test("initializes zaplib and calls sum", async () => {
  // Read and compile wasm module
  const wasmPath = "path to wasm module file";
  const wasmModule = WebAssembly.compile(fs.readFileSync(wasmPath));

  await zaplib.initialize({ wasmModule });

  // Test "sum" call
  const buffer = new SharedArrayBuffer(8);
  const data = new Uint8Array(buffer);
  data.set([1, 2, 3, 4, 5, 6, 7, 8]);
  const [result] = await zaplib.callRust("sum", [data]);
  expect(result).toBe("36");
});
```
Couple of notes:
 - Zaplib provides a set of polyfills for running in Node.js, which can be found in `zaplib/dist/zaplib_nodejs_polyfill` (in the `zaplib` npm package).
 - Make sure to initialize `wasmPath = ...` to the path of your wasm file
