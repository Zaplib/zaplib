# Jest Integration

Zaplib can run in the [Jest testing framework](https://jestjs.io/). Following [Tutorial: Integrating with JS](./tutorial_js_rust_bridge.md), let's assume we have a `sum` hook for `callRustAsync` defined:

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
  const [result] = await zaplib.callRustAsync("sum", [data]);
  expect(result).toBe("36");
});
```
Couple of notes:
 - Zaplib provides a set of polyfills for running in Node.js, which can be found in `zaplib/dist/zaplib_nodejs_polyfill` (in the `zaplib` npm package).
 - Make sure to initialize `wasmPath = ...` to the path of your wasm file
