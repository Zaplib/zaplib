# Web Workers
Zaplib can also be used inside of your own Web Workers. This comes with both some extra features, as well as some caveats.

First, include the Web Worker entry point (`zaplib_worker_runtime.development.js`).

Note that when using [Zapium](./zapium.md) we don't support any of these functions yet.

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

* Zaplib-managed typed arrays are those returned by `zaplib.createReadOnlyBuffer`, `zaplib.callRustSync`, and so on.
* When sending a Zaplib-managed over `postMessage`, just wrap it in `zaplib.serializeZapArrayForPostMessage()`.
* On the other side of the `postMessage` interface, get back a Zaplib-managed typed array by calling `zaplib.deserializeZapArrayFromPostMessage()`.
* Both of these methods are synchronous.
