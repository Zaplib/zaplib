# Basics

First, make sure to install the [npm package](https://www.npmjs.com/package/zaplib) and include the main entry point (`zaplib_runtime.js`).

## zaplib.initialize

This initializes the library. A couple of things happen:
* The `.wasm` file is downloaded and compiled.
* A Web Worker is created for the main Rust event loop.
* A `<canvas>` element spanning the entire page is created and added to `<body>`. It is transparent and doesn't respond to input events except when actively doing rendering within Rust. But if you need to fully hide it or override styles, use the `.zaplib_canvas` CSS class.
* A `<textarea>` element is added to `<body>`. Again, it only springs into action when necessary. But if you need to fully hide it, use the `.zaplib_textarea` CSS class.
* We add event listeners on the full page to capture events that are relevant for Zaplib.
* We monkey-patch typed array constructors (e.g. `new Uint8Array`) and `postMessage` calls to add some additional features. See [next chapter](./bridge_api_params.md) for more details.

| Parameter (Typescript)                      | Description |
|---------------------------------------------|---------|
| `initParams.filename: string` | Path to the `.wasm` file. During development, typically something like `/target/wasm32-unknown-unknown/debug/my_package_name.wasm`. |
| `initParams.defaultStyles?: boolean` | Whether to inject some default styles, including a loading indicator and a canvas. Useful for examples / getting started. |
| `initParams.canvas?: HTMLCanvasElement` | A `<canvas>` element that must span the whole page. If not given, then rendering isn't possible. `defaultStyles: true` will automatically create this and add it to `<body>`. See also the [Canvas page](./rendering_api_canvas.md). |

<p></p>

| Returns (Typescript)                       | Description |
|---------------------------------------------|---------|
| `Promise<void>`                           | Promise that resolves when you can call other functions. |

**Caveats**
* Can only be called on the browser's main thread; in a worker use `zaplib.initializeWorker()`.
* `filename` is ignored in CEF.

## zaplib.callRust

Calls Rust with some parameters. The Rust code gets executed inside the main Rust Web Worker.

| Parameter (Typescript)                      | Description |
|---------------------------------------------|---------|
| `name: string`                              | Some descriptive name of what you want to call. |
| <code>params?: (Uint8Array \| Float32Array \| string)[]</code> | Array of parameters. See [next chapter](./bridge_api_params.md) for more details. |

<p></p>

| Returns (Typescript)                       | Description |
|---------------------------------------------|---------|
| <code>Promise<(Uint8Array \| Float32Array \| string)[]></code> | Return parameters. Typed arrays are backed by the WebAssembly memory, and are zero-copy. Strings are always copied. |

On the Rust side, define a function to handle `callRust` calls using the `register_call_rust!()` macro:

```rust,noplayground
fn call_rust(name: String, params: Vec<ZapParam>) -> Vec<ZapParam> { ... }
register_call_rust!(call_rust);
```

Or if you have an application struct which you need access to, use `cx.on_call_rust()`:

```rust,noplayground
impl ExampleApp {
    fn new(cx: &mut Cx) -> Self {
        cx.on_call_rust(Self::on_call_rust);
        Self {}
    }

    fn on_call_rust(
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

Use these functions to allocate raw data on the WebAssembly heap. These are convenience functions that have the same effect as calling `zaplib.callRust` with non-Zaplib-backed typed arrays and immediately returning them.

Note that when called on the browser's main thread, these calls are asynchronous (they return a `Promise`), while within Web Workers they are synchronous. In the future, we would like to make them synchronous in both cases.

## zaplib.registerCallJsCallbacks & zaplib.unregisterCallJsCallbacks

In order to call JS from Rust — e.g. in response to an event — you can register callbacks on the JS side, using `zaplib.registerCallJsCallbacks`. An example:

```js
zaplib.registerCallJsCallbacks({
    log(params) {
        console.log("log fn called", params[0]);
    },
});
```

Then, in Rust, use: `cx.call_js("log", vec!["Hello, World!".to_string().into_param()])`, similarly to returning params from `call_rust`.

Currently these calls are one-way; it is not possible to directly return values. In order to do that, make a separate call to `zaplib.callRust`.

In order to unregister callbacks, use e.g. `zaplib.unregisterCallJsCallbacks(["log"]);`.
