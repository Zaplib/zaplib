# API Overview

This is an overview of the different APIs for communicating between JavaScript and Rust.

The [Zaplib package](https://www.npmjs.com/package/zaplib) on npm has two entrypoints:
1. `zaplib_runtime.js`: the main runtime, to be used on the browser's main thread.
2. `zaplib_worker_runtime.js`: the Web Worker runtime, for use in your workers.

The APIs between these runtimes is mostly the same, but there are some small differences which we will note.

Here is an overview of all the JS APIs, and their support with the WebAssembly runtime and the experimental [CEF](./cef.md) runtime.  Missing features are annotated with their ticket id.

| API                                         | Browser main thread | Browser Web Worker | [CEF](./cef.md) main thread | [CEF](./cef.md) Web Worker |
| ------------------------------------------- | :---------------: | :---------------: | :--------------: | :--------------: |
| zaplib.initialize                           |       ✅          |        n/a          |       ✅       |       n/a         |
| zaplib.initializeWorker                     |      n/a          |        ✅          |       n/a       |    [#69][2] |
| zaplib.registerCallJsCallbacks              |       ✅          |      [#70][3]      |       ✅        |  [#69][2]  [#70][3] |
| zaplib.unregisterCallJsCallbacks            |       ✅          |      [#70][3]      |       ✅        |  [#69][2]  [#70][3] |
| zaplib.callRust                             |       ✅          |        ✅          |       ✅        |   [#69][2] |
| zaplib.createReadOnlyBuffer                 |       ✅          |        ✅          |       ✅        |   [#69][2] |
| zaplib.createMutableBuffer                  |       ✅          |        ✅          |       ✅        |   [#69][2] |
| zaplib.callRustSync                         |       ✅          |        ✅          |       ✅        |   [#69]`[2] |
| zaplib.newWorkerPort                        |       ✅          |        ✅          |     [#69][2]    |   [#69][2] |
| zaplib.serializeZapArrayForPostMessage      |       ✅          |        ✅          |     [#69][2]    |   [#69][2] |
| zaplib.deserializeZapArrayFromPostMessage   |       ✅          |        ✅          |     [#69][2]    |   [#69][2] |
| zaplib.jsRuntime                            |       ✅          |      [#69][2]      |       ✅        |   [#69][2] |

[1]: https://github.com/Zaplib/zaplib/issues/51
[2]: https://github.com/Zaplib/zaplib/issues/69
[3]: https://github.com/Zaplib/zaplib/issues/70
