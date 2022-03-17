# Tutorial: Sharing Data

This guide is a followup to the [Tutorial: Integrating with JS](./tutorial_js_rust_bridge.md). It will show you how to avoid copying data when calling across the JavaScript-Rust boundary.

## Identifying a need
Let's start with our example from before, with a few modifications. We still want to calculate a sum in WebAssembly, but now we also want to calculate the product using a separate function call.
```js
// index.js (after zaplib.initialize)
const values = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
const [sumArray] = zaplib.callRustSync('sum', [values]);
const sum = sumArray[0];
document.getElementById('root').textContent = sum;
```
Like in our last guide, this is a contrived example, but one that illustrates a pitfall when repeatedly calling Rust with an input buffer.

Since the input buffer is stored in memory separate from WebAssembly, every call will re-copy it so that our Rust code can read the values. For large enough arrays, this can lead to a significant slowdown.

Zaplib helps you solve this problem by giving you read and write access to Rust-managed memory.

## Allocating memory in Rust
Let's first create a Uint8Array that's managed in Rust. Our new code:
```js
// index.js (after zaplib.initialize)
const values = zaplib.createReadOnlyBuffer(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
const [sumArray] = zaplib.callRustSync('sum', [values]);
const sum = sumArray[0];
document.getElementById('root').textContent = sum;
```

### What's new?
We only change one line above: initializing `values` using `zaplib.createReadOnlyBuffer`. This consumes a `Uint8Array` and copies it into WebAssembly memory, which is Rust-managed.

## Reusing the allocated memory.
Let's add to our contrived example, and get both the sum and the product of the values, using two separate calls to `callRustSync`:

```rust,noplayground
// src/main.rs
use zaplib::*;

fn sum(values: &[u8]) -> u8 {
    values.iter().sum()
}

fn product(values: &[u8]) -> u8 {
    values.iter().product()
}

fn call_rust(name: String, params: Vec<ZapParam>) -> Vec<ZapParam> {
    if name == "sum" {
        let values = params[0].as_u8_slice();
        let response = vec![sum(&values)].into_param();
        return vec![response];
    } else if name == "product" {
        let values = params[0].as_u8_slice();
        let response = vec![product(&values)].into_param();
        return vec![response];
    }
    vec![]
}

register_call_rust!(call_rust);
```

```js
// index.js (after zaplib.initialize)
const values = zaplib.createReadOnlyBuffer(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
const sum = zaplib.callRustSync('sum', [values])[0][0];
const product = zaplib.callRustSync('product', [values])[0][0];
document.getElementById('root').textContent = "sum: " + sum + " product: " + product;
```

Even though we called `callRustSync` multiple times with `values`, there was no copying of data involved!

## Read-Only vs Mutable
The `values` buffer is read-only, which means that you can safely read from it in JavaScript and Rust at the same time! In fact, you can pass it safely to Rust threads or Web Workers (using `zaplib.serializeZapArrayForPostMessage`). Zaplib will keep track of where you use the array, so that it gets properly deallocated when you don't use it anymore.

Note that we currently don't enforce that you don't mutate `values`. There is no built-in way in JavaScript to do that. (In the future we might run periodic checksums on the data in debug builds, to prevent bugs.)

To mutate the data on the JavaScript side, the easiest way is to make a copy of `values` and call `zaplib.createReadOnlyBuffer` again. On the Rust side, you can use `let new_vec = values.as_vec();` to copy into a new `Vec<u8>`, and then return that using `new_vec.into_param()`.

It is also possible to mutate data on either side without copying, but that is a more advanced technique with we'll cover in a future tutorial.
