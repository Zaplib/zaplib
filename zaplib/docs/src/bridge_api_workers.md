# Web Workers
Zaplib can also be used inside of your own Web Workers. This comes with both some extra features, as well as some caveats.

First, include the Web Worker entry point (`zaplib_worker_runtime.js`).

Note that when using CEF we don't support any of these functions yet.

## zaplib.newWorkerPort & zaplib.initializeWorker
In order to use Zaplib inside Web Workers, we first have to create a "worker port" on the main thread, using `zaplib.newWorkerPort()`. Send that port to the Web Worker using whatever `postMessage` mechanism you already use. Be sure to add the port to the list of transferables. Example:

```js
const zapWorkerPort = zaplib.newWorkerPort();
myWorker.postMessage(zapWorkerPort, [zapWorkerPort]);
```

Within the Web Worker, receive this port, and call `zaplib.initializeWorker(zapWorkerPort)`. Just like `zaplib.initialize` this returns a `Promise` indicating when you can call other functions on `zaplib`. Under the hood, we do the following:
* A cached, compiled version of the main `.wasm` file is loaded and instantiated.
* A thread-specific stack and thread-local storage are allocated and initialized.
* Shared WebAssembly memory is mounted.

In the worker, the code would look something like this:
```js
self.onmessage = function(e) {
    const zapWorkerPort = e.data;
    zaplib.initializeWorker(zapWorkerPort).then(() => {
        // actual code here.
    });
};
```

## zaplib.serializeZapArrayForPostMessage & zaplib.deserializeZapArrayFromPostMessage

When a Zaplib-managed typed array gets garbage collected, the WebAssembly memory is updated accordingly (the refcount is decreased for read-only buffers; and the memory is freed for mutable buffers). This does mean that if you want to pass such a typed array to a Web Worker, that you have to use `zaplib.serializeZapArrayForPostMessage`. If you don't, the data might get de- or re-allocated before you can use it.

Note that this *is* enforced by monkey-patching `postMessage` when you call `zaplib.initialize()` or `zaplib.initializeWorker`, so don't worry about getting this wrong.

* Zaplib-managed typed arrays are those returned by `zaplib.createReadOnlyBuffer`, `zaplib.callRust`, and so on.
* When sending a Zaplib-managed over `postMessage`, just wrap it in `zaplib.serializeZapArrayForPostMessage()`.
* On the other side of the `postMessage` interface, get back a Zaplib-managed typed array by calling `zaplib.deserializeZapArrayFromPostMessage()`.
* Both of these methods are synchronous.

## zaplib.callRustInSameThreadSync

In Web Workers we also support calling Rust within that very thread. This means that execution transfers from JS to Rust, and no other processing can happen until the function returns. It also means that no `Promise`s are involved; it's purely synchronous code.

To register a callback, you have to use `cx.on_call_rust_in_same_thread_sync()`. However, the callback function has no access to the application struct, nor to `Cx` itself:

```rust,noplayground
impl ExampleApp {
    fn new(cx: &mut Cx) -> Self {
        cx.on_call_rust_in_same_thread_sync(Self::on_call_rust_in_same_thread_sync);
        Self {}
    }

    fn on_call_rust_in_same_thread_sync(
        name: String,
        params: Vec<ZapParam>
    ) -> Vec<ZapParam> {}
}
```

On the JS side, call `zaplib.callRustInSameThreadSync()`. This has the same function signature as `zaplib.callRust`, except that its results are not wrapped in a `Promise`.
