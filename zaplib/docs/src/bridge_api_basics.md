# Basics

First, make sure to include the main entry point (`zaplib_runtime.development.js`).

## zaplib.initialize

This initializes the library. A couple of things happen:
* The `.wasm` file is downloaded and compiled.
* A Web Worker is created for the main Rust event loop.
* A `<canvas>` element spanning the entire page is created and added to `<body>`. It is transparent and doesn't respond to input events except when actively doing rendering within Rust. But if you need to fully hide it or override styles, use the `.zaplib_canvas` CSS class.
* A `<textarea>` element is added to `<body>`. Again, it only springs into action when necessary. But if you need to fully hide it, use the `.zaplib_textarea` CSS class.
* We add event listeners on the full page to capture events that are relevant for Zaplib.
* We monkey-patch typed array constructors (e.g. `new Uint8Array`) and `postMessage` calls to add some additional features. See [next chapter](./bridge_api_params.md) for more details.
* Call the convenience method `zaplib.isInitialized` to check for the initialization status. Once set to true, it will never go back to false (even in case of an error).

| Parameter (Typescript)                      | Description |
|---------------------------------------------|---------|
| <code>initParams.wasmModule: string &#124; Promise<WebAssembly.Module></code> | Path to the `.wasm` file or a Promise for compiled wasm module. During development, typically something like `/target/wasm32-unknown-unknown/debug/my_package_name.wasm`. |
| `initParams.defaultStyles?: boolean` | Whether to inject some default styles, including a loading indicator, text area and a canvas. Useful for examples / getting started. |
| `initParams.canvas?: HTMLCanvasElement` | A `<canvas>` element that must span the whole page. If not given, then rendering isn't possible. `defaultStyles: true` will automatically create this and add it to `<body>`. See also the [Canvas page](./rendering_api_canvas.md). |
| `initParams.createTextArea?: boolean` | Whether to create a hidden text area element that is used when entering input in Rust |
| `initParams.onPanic?: (e: Error) => void` | A callback to run if Zaplib panics during `draw` or `handle` functions. |

<p></p>

| Returns (Typescript)                       | Description |
|--------------------------------------------|---------|
| `Promise<void>`                            | Promise that resolves when Zaplib has been initialized and you can call other Zaplib functions. |

**Caveats**
* Can only be called on the browser's main thread; in a worker use `zaplib.initializeWorker()`.
* `wasmModule` is ignored in [Zapium](./zapium.md).
* Call `zaplib.close` when you want to terminate all the Web Workers Zaplib opens. This can be useful when running tests.

## zaplib.callRustSync

We support calling Rust synchronously. This means that execution transfers from JS to Rust, and no other processing can happen until the function returns. It also means that no `Promise`s are involved; it's purely synchronous code.

| Parameter (Typescript)                      | Description |
|---------------------------------------------|---------|
| `name: string`                              | Some descriptive name of what you want to call. |
| <code>params?: (Uint8Array \| Float32Array \| string)[]</code> | Array of parameters. See [next chapter](./bridge_api_params.md) for more details. |

<p></p>

| Returns (Typescript)                     | Description |
|---------------------------------------------|---------|
| <code>(Uint8Array \| Float32Array \| string)[]</code> | Return values. Typed arrays are backed by the WebAssembly memory, and are zero-copy. Strings are always copied. |

To register a callback, you have to use `cx.on_call_rust_sync()`. However, the callback function has no access to the application struct, nor to `Cx` itself:

```rust,noplayground
impl ExampleApp {
    fn new(cx: &mut Cx) -> Self {
        cx.on_call_rust_sync(Self::on_call_rust_sync);
        Self {}
    }

    fn on_call_rust_sync(
        name: String,
        params: Vec<ZapParam>
    ) -> Vec<ZapParam> {}
}
```

On the JS side, call `zaplib.callRustSync`. This has the same function signature as `zaplib.callRustAsync`, except that its results are not wrapped in a `Promise`.

## zaplib.callRustAsync

Calls Rust with some parameters. The Rust code gets executed inside the main Rust Web Worker.

| Parameter (Typescript)                      | Description |
|---------------------------------------------|---------|
| `name: string`                              | Some descriptive name of what you want to call. |
| <code>params?: (Uint8Array \| Float32Array \| string)[]</code> | Array of parameters. See [next chapter](./bridge_api_params.md) for more details. |

<p></p>

| Returns (Typescript)                     | Description |
|---------------------------------------------|---------|
| <code>Promise<(Uint8Array \| Float32Array \| string)[]></code> | Return values in a `Promise`. Typed arrays are backed by the WebAssembly memory, and are zero-copy. Strings are always copied. |

On the Rust side, define a function to handle these calls using the `register_call_rust!()` macro:

```rust,noplayground
fn call_rust(name: String, params: Vec<ZapParam>) -> Vec<ZapParam> { ... }
register_call_rust!(call_rust);
```

Or if you have an application struct which you need access to:

```rust,noplayground
impl ExampleApp {
    fn new(cx: &mut Cx) -> Self {
        cx.on_call_rust_async(Self::on_call_rust_async);
        Self {}
    }

    fn on_call_rust_async(
        &mut self,
        cx: &mut Cx,
        name: String,
        params: Vec<ZapParam>
    ) -> Vec<ZapParam> {}
}
```

`ZapParam` matches the type of parameter that was pass in on the JS side. Get out the actual data using an `as_*` or `into_*` helper function. Similarly, return data by turning it into a `ZapParam` using the `into_param` on a supported type. For example, for converting to and from `String`s:

```rust,noplayground
fn call_rust(name: String, params: Vec<ZapParam>) -> Vec<ZapParam> {
    // Converting to a string, and printing it:
    log!("String value: {}", params[0].as_str());
    return vec!["Return value".to_string().into_param()];
}
```

For more information about the parameter types, see the [next chapter](./bridge_api_params.md).

## zaplib.createReadOnlyBuffer & zaplib.createMutableBuffer

Use these functions to allocate raw data on the WebAssembly heap. These are convenience functions that have the same effect as calling `zaplib.callRustSync` with non-Zaplib-backed typed arrays and immediately returning them.

## zaplib.isZapBuffer

Determines if a given ArrayBuffer is backed by Zaplib managed memory. This can be especially useful when determining how to communicate a buffer across a WebWorker boundary - [see this section](/docs/bridge_api_workers.html#zaplibserializezaparrayforpostmessage--zaplibdeserializezaparrayfrompostmessage).

| Parameter (Typescript) | Returns (Typescript) | Description |
|-|-|-|
| `buffer: ArrayBufferLike` | `boolean` | True if Zaplib managed memory, false if JavaScript managed memory. |

## zaplib.registerCallJsCallbacks & zaplib.unregisterCallJsCallbacks

In order to call from Rust to JS — e.g. in response to an event in Rust — you can register callbacks on the JS side, using `zaplib.registerCallJsCallbacks`. An example:

```js
zaplib.registerCallJsCallbacks({
    log(params) {
        console.log("log fn called", params[0]);
    },
});
```

Then, in Rust, use: `cx.call_js("log", vec!["Hello, World!".to_string().into_param()])`, similarly to returning params from `call_rust`.

Currently these calls are one-way; it is not possible to directly return values. In order to do that, make a separate call to `zaplib.callRustSync` or `zaplib.callRustAsync`.

In order to unregister callbacks, use e.g. `zaplib.unregisterCallJsCallbacks(["log"]);`.
