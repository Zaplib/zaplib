# Known Issues

Zaplib is still in its early days, and there are quite a few things to stabilize. Here are some of the known issues.

**Overall**
* Few automated tests yet (except for JS<=>Wasm bridge and some font stuff).

**Wasm runtime**
* `test_multithread` deadlocks in Chrome after a while (not sure about other browsers).
* Safari 15.2 `test_multithread` doesn't work well at all (even after [this bugfix](https://bugs.webkit.org/show_bug.cgi?id=234833)).
* Touch is not super well tested/supported yet.
* Memory initialization in Mobile Safari is not working well (often doesn't allocate enough memory); see also [this thread](https://github.com/WebAssembly/design/issues/1397).
* Threads leak memory since we never deallocate the TLS/shadow stack (see also [this issue](https://github.com/rust-lang/rust/issues/77839)).
* Error handling is confusing; a panic can cause the console to get flooded with unrelated errors afterwards.

**JS<=>Wasm bridge**
* Issues with capturing/preventing mouse events (e.g. right click).
* No enforcement of buffer constraints (e.g. read-only; no use after moving ownership back to Rust).

**Rendering**
* Some memory leakage / wastage of CPU/GPU buffers.
* Resizing the window can be janky/laggy.
* Layouts can be confusing/buggy.
* 2d rendering API doesn't fully match HTML 2d canvas behavior (though we have to decide what level of discrepancy we're okay with).

**OSX native**
* Redrawing seems to leak a lot of memory.

**OSX CEF**
* Stuck on old version (because we only got single process working on an old version).
* Missing APIs compared to JS<=>Wasm bridge.
* Too many memory copies.
* Missing support for layering Rust rendering behind web rendering (only on top is supported).

**Win/Linux**
* Bunch of missing APIs.
* Not well-tested.
