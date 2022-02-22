# Types of Parameters

These are all the types of parameters that can be sent over the JS-Rust bridge:
<table>
<thead><tr><td colspan="2">ZapParam::String</td></tr></thead>
<tr><td colspan="2"><em>Most common for small JSON-serialized data (e.g. using <a href="https://serde.rs/">Serde</a>).</em></td></tr>
<tr><td>Create in JS</td><td><code>"Hello, world!"</code></td></tr>
<tr><td>Type in Rust</td><td><code>String</code></td></tr>
<tr><td>Returned to JS</td><td><code>String</code></td></tr>
<tr><td>Borrowing data</td><td><code>as_str() -> &str</code></td></tr>
<tr><td>Transferring ownership</td><td><code>into_string() -> String</code></td></tr>
<tr><td>Caveats</td><td>Data is copied when passed over the bridge.</td></tr>
<thead><tr><td colspan="2">ZapParam::ReadOnlyU8Buffer</td></tr></thead>
<tr><td colspan="2"><em>Most common for large serialized data.</em></td></tr>
<tr><td>Create in JS</td><td><code>zaplib.createReadOnlyBuffer(new Uint8Array([1, 2, 3]))</code></td></tr>
<tr><td>Type in Rust</td><td><code>Arc&lt;Vec&lt;u8>></code></td></tr>
<tr><td>Returned to JS</td><td><code>ZapTypedArray (extends Uint8Array)</code></td></tr>
<tr><td>Borrowing data</td><td><code>as_u8_slice() -> &[u8]</code></td></tr>
<tr><td>Adding ownership (refcount)</td><td><code>as_arc_vec_u8() -> Arc&lt;Vec&lt;u8>></code></td></tr>
<tr><td>Caveats</td><td>No enforcement of read-only on JS side (yet).</td></tr>
<thead><tr><td colspan="2">ZapParam::ReadOnlyF32Buffer</td></tr></thead>
<tr><td colspan="2"><em>Most common for graphics data.</em></td></tr>
<tr><td colspan="2">Same as above, but instead with <code>f32</code> and <code>Float32Array</code>.</td></tr>
<thead><tr><td colspan="2">ZapParam::MutableU8Buffer</td></tr></thead>
<tr><td colspan="2"><em>Less common.</em></td></tr>
<tr><td>Create in JS</td><td><code>zaplib.createMutableBuffer(new Uint8Array([1, 2, 3]))</code></td></tr>
<tr><td>Type in Rust</td><td><code>Vec&lt;u8></code></td></tr>
<tr><td>Returned to JS</td><td><code>ZapTypedArray (extends Uint8Array)</code></td></tr>
<tr><td>Borrowing data</td><td><code>as_u8_slice() -> &[u8]</code></td></tr>
<tr><td>Transferring ownership</td><td><code>into_vec_u8() -> Vec&lt;u8></code></td></tr>
<tr><td>Caveats</td><td>Once passed from JS to Rust, the data cannot be used on the JS side any more (neither reading nor writing); representing transfer of ownership to Rust. This is not enforced (yet).</td></tr>
<thead><tr><td colspan="2">ZapParam::MutableF32Buffer</td></tr></thead>
<tr><td colspan="2"><em>Less common.</em></td></tr>
<tr><td colspan="2">Same as above, but instead with <code>f32</code> and <code>Float32Array</code>.</td></tr>
</table>

As noted in the caveats above, you must take care when using these buffers on the JavaScript side:
* **Read-only buffers should not be mutated in JS.** If you do mutate them anyway, race conditions and data corruption can occur. This restriction is not enforced (yet).
* Mutable buffers can be mutated in JS. However, **once you pass a mutable buffer into Rust, you cannot use the buffer in JS in *any way*.** This is because ownership is passed to Rust, which can now mutate the data. If you read from such a stale buffer in JS, race conditions and data corruption can occur. This restriction is not enforced (yet).
  * It is possible to mutate some data in JS, then in Rust, and then in JS again, without ever copying of the data. Just pass the mutable buffer back from Rust to JS when you're done with it.

When a `u8` or `f32` buffer is returned to JS, you get a `ZapTypedArray`:
* This extends either `Uint8Array` or `Float32Array`.
* This typed array is backed by the WebAssembly memory.
* When the typed array gets garbage collected, the WebAssembly memory is updated accordingly (the refcount is decreased for read-only buffers; and the memory is freed for mutable buffers).
* This does mean that if you want to pass such a typed array to a Web Worker, that you have to use `zaplib.serializeZapArrayForPostMessage`. If you don't, the data might get de- or re-allocated before you can use it.
  * This *is* enforced by monkey-patching `postMessage` when you call `zaplib.initialize()`, so don't worry too much about it.
* It is possible to call [`subarray()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray/subarray) on these typed arrays, and the garbage collection be tracked properly.
  * It is even possible to create a new typed array using `new Uint8Array(zapArray.buffer, zapArray.byteOffset, zapArray.length)`, and the garbage collection will still be tracked properly!
  * This makes it possible to pass these typed arrays into most existing libraries.
  * However, it's not possible to pass a sub-slice of a typed array to Rust.

When sending small amounts of data in either direction, we recommend simply JSON-serializing the data and sending it as a string. On the Rust side, [Serde](https://serde.rs/) is a fine library for this.

Futher note that when using [CEF](./cef.md), data is often copied anyway, even when in the WebAssembly version it is not. This is one of the reasons why we do not recommend using [CEF](./cef.md) yet.
