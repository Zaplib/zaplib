# API Overview

This API Overview highlights some of the the Zaplib Standard Library. Our [API Reference](/target/doc/zaplib/index.html) is is more comprehensive but isn't as user friendly.
## Universal Interfaces

Some Rust functions, particularly around IO, don't work in WebAssembly out-of-the-box. Zaplib provides a number of cross-platform, "universal" interfaces that work both natively and in WebAssembly.


| Rust | Universal | |
|----------|---------------|-------|
| [`println!`](https://doc.rust-lang.org/std/macro.println.html) | [`log!`](/target/doc/zaplib/macro.log.html) | Logs to the console (with line number). |
| [`thread`](https://doc.rust-lang.org/std/thread/) | [`universal_thread`](/target/doc/zaplib/universal_thread/index.html) | <ul><li><code><a href="/target/doc/zaplib/universal_thread/fn.spawn.html">spawn</a></code> (without <code><a href="https://doc.rust-lang.org/std/thread/struct.JoinHandle.html">JoinHandle</a></code>)</li><li><code><a href="/target/doc/zaplib/universal_thread/fn.sleep.html">sleep</a></code></li><li>We recommend using a thread pool, e.g. the <a href="https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html#method.spawn_handler">rayon's <code>ThreadPoolBuilder</code></a>.</li></ul> |
| [`Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html) | [`UniversalInstant`](/target/doc/zaplib/universal_instant/struct.UniversalInstant.html) | `elapsed, now, duration_since, checked_add, checked_sub, +, -, +=, -=` |
| [`File`](https://doc.rust-lang.org/stable/std/fs/struct.File.html) | [`UniversalFile`](/target/doc/zaplib/universal_file/struct.UniversalFile.html) | <ul><li><code><a href="/target/doc/zaplib/universal_file/struct.UniversalFile.html#method.open">open</a></code> (on WebAssembly this blocks until the whole file is loaded in memory)</li><li><code><a href="/target/doc/zaplib/universal_file/struct.UniversalFile.html#method.open_url">open_url</a></code> (non-standard; load an absolute URL)</li><li><code><a href="/target/doc/zaplib/universal_file/struct.UniversalFile.html#method.clone">clone</a></code> (cheap; clones just a handle to the data; doesn't preserve cursor)</li><li><code><a href="https://doc.rust-lang.org/std/io/trait.Read.html">std::io::Read</a></code></li><li><code><a href="https://doc.rust-lang.org/std/io/trait.Seek.html">std::io::Seek</a></code></li><li><code><a href="/target/doc/zaplib/read_seek/trait.ReadSeek.html">ReadSeek</a></code> (non-standard; convenient trait for <code>Read + Seek</code>)</li></ul> |
| non-standard | [`universal_http_stream`](/target/doc/zaplib/universal_http_stream/index.html) | <ul><li><code><a href="/target/doc/zaplib/universal_http_stream/fn.request.html">request</a></code> (returns data as it comes in; useful for large files)</li><li><code><a href="https://doc.rust-lang.org/std/io/trait.Read.html">std::io::Read</a></code></li></ul> |
| non-standard | [`universal_rand`](/target/doc/zaplib/universal_rand/index.html) | [`random_128`](/target/doc/zaplib/universal_rand/fn.random_128.html) |

## `Cx` & Events

[`Cx`](/target/doc/zaplib/cx/struct.Cx.html) object. This is a global "context" object, that gets passed around practically everywhere.

### Construction Event

When the app is constructed, a [`Construct`](/target/doc/zaplib/enum.Event.html#variant.Construct) event is fired. It is fired exactly once, and before any other calls to `handle` or `draw`. The event contains no further information.

### Timers

Calling [`cx.start_timer`](/target/doc/zaplib/struct.Cx.html#method.start_timer) creates a new [`Timer`](/target/doc/zaplib/struct.Timer.html) object. When the timer fires, a [`TimerEvent`](/target/doc/zaplib/struct.TimerEvent.html) event is dispatched. Use [`timer.is_timer`](/target/doc/zaplib/struct.Timer.html#method.is_timer) to check if that event belongs to a particular timer. Use [`cx.stop_timer`](/target/doc/zaplib/struct.Cx.html#method.stop_timer) to stop it.

### Signals

Signals are user-defined events that can be used for anything you want. Create a new [`Signal`](/target/doc/zaplib/struct.Signal.html) object by calling [`cx.new_signal`](/target/doc/zaplib/struct.Cx.html#method.new_signal). Then send it with a [`StatusId`](/target/doc/zaplib/type.StatusId.html) using [`cx.send_signal`](/target/doc/zaplib/struct.Cx.html#method.send_signal) (same thread) or [`Cx::post_signal`](/target/doc/zaplib/struct.Cx.html#method.post_signal) (any thread). This will trigger a [`SignalEvent`](/target/doc/zaplib/struct.SignalEvent.html) on the main thread (`handle` and `draw` are always called on the main Rust thread).

Note that the Signals API is a bit complicated currently; we aim to improve this so you can send any user-defined events.

### WebSockets

[`cx.websocket_send`](/target/doc/zaplib/struct.Cx.html#method.websocket_send) sends a message on a WebSocket. If no WebSocket yet exists for the given URL, a new one is opened. When receiving a message on a WebSocket, a [WebSocketMessageEvent](/target/doc/zaplib/struct.WebSocketMessageEvent.html) is fired.

### Focus

If the browser tab or native window gets or loses focus, then [`AppFocus`](/target/doc/zaplib/enum.Event.html#variant.AppFocus) or [`AppFocusLost`](/target/doc/zaplib/enum.Event.html#variant.AppFocusLost) are fired, respectively.

### User files

To create a drop target for the entire window / browser tab, we have to create a [`Window`](/target/doc/zaplib/struct.Window.html) with [`create_add_drop_target_for_app_open_files`](/target/doc/zaplib/struct.Window.html#structfield.create_add_drop_target_for_app_open_files). Then, when dropping a file, an [`AppOpenFilesEvent`](/target/doc/zaplib/struct.AppOpenFilesEvent.html) event will fire.

There are also events for when a file drag is [started](/target/doc/zaplib/enum.Event.html#variant.FileDragBegin), [updated](/target/doc/zaplib/enum.Event.html#variant.FileDragUpdate), or [cancelled](/target/doc/zaplib/enum.Event.html#variant.FileDragCancel).

### Profiling

Basic profiling using the console can be done using [`cx.profile_start`](/target/doc/zaplib/struct.Cx.html#method.profile_start) and [`cx.profile_end`](/target/doc/zaplib/struct.Cx.html#method.profile_end).

## Missing compatibility

Some standard library APIs don't work in all contexts. APIs that are currently unsupported in a context are annotated with a tracking ticket ID.

| API                                         | Rust main thread  | Rust child thread  | JS main thread   | JS WebWorker     |
| ------------------------------------------- | :---------------: | :---------------:  | :--------------: | :--------------: |
| Logging (`log!`)                            |       ✅          |        ✅          |        ✅         |       ✅        |
| Spawning threads (`universal_thread`)       |       ✅          |        ✅          |     [#72][2]      |     [#72][2]    |
| Current time (`UniversalInstant`)           |       ✅          |        ✅          |        ✅         |       ✅        |
| Reading local files (`UniversalFile`)       |       ✅          |        ✅          | [#72][2] [#66][4] |     [#72][2]    |
| Writing local files                         |    [#73][3]       |     [#73][3]       | [#73][3] [#66][4] |     [#73][3]    |
| HTTP requests (`UniversalFile`/`universal_http_stream`) |     ✅        |      ✅    |      [#66][4]     |     ✅      |
| Random (`universal_rand`)                   |       ✅          |        ✅          |        ✅         |       ✅        |
| Websockets (`cx.websocket_send`)            |       ✅          |        [#71][1]    |     [#71][1]     |    [#71][1]    |
| Timers (`cx.start_timer`)                   |       ✅          |        [#71][1]    |     [#71][1]     |    [#71][1]    |
| Posting signals (`Cx::post_signal`)         |       ✅          |        ✅          |     [#72][2]      |     [#72][2]    |
| Profiling (`cx.profile_start`)              |       ✅          |        [#71][1]    |     [#71][1]     |    [#71][1]    |
| Blocking Rust threading primitives ([`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)) | ✅ | ✅ | [#66][4] | ✅

[1]: https://github.com/Zaplib/zaplib/issues/71
[2]: https://github.com/Zaplib/zaplib/issues/72
[3]: https://github.com/Zaplib/zaplib/issues/73
[4]: https://github.com/Zaplib/zaplib/issues/66
