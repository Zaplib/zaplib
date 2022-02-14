# Tutorial: Hello World Console

Let's write the most basic application: printing "Hello, world!" to the console.

Either create a new folder in `zaplib/examples`, or follow along with the existing `tutorial_hello_world_console`.

First, let's create our `Cargo.toml`:

```toml
{{#include ../../examples/tutorial_hello_world_console/Cargo.toml}}
```

Now, let's create `src/main.rs`:

```rust,noplayground
{{#include ../../examples/tutorial_hello_world_console/src/main.rs}}
```

Let's break it down a bit. The app must be a `struct` that implement three methods:
* `new` — Returns an initialized struct and any initial state we add. For now, let's call the `default` implementation.
* `handle` — An entrypoint into Zaplib's event handling system. We will go in depth on various event types in a different tutorial. For now, we'll put our `log!()` call in the the `Construct` event.
* `draw` — Called when requesting a draw. This will control what gets shown on the application window, which we don't use yet.

The call to `main_app!()` tells Zaplib to use the `App` struct for all its eventing and rendering.

This is already enough to run the native version: `cargo run -p tutorial_hello_world_console`. Hurray! It prints "Hello, world!".

Notice how this program currently never exits on its own. That behavior is similar to the web version, where the program doesn't exit until the browser window is closed. In our case here we don't have a native window yet, so terminate the program using CTRL+C.

### WebAssembly

Now let's add an `index.html`:

```html
{{#include ../../examples/tutorial_hello_world_console/index.html}}
```

Compile to WebAssembly: `./zaplib/scripts/build_website_dev.sh -p tutorial_hello_world_console` (or whatever you named your folder)

Be sure to run the server, as described in [Getting Started](./getting_started.md).

Navigate to [http://localhost:3000/zaplib/examples/tutorial_hello_world_console](http://localhost:3000/zaplib/examples/tutorial_hello_world_console), open the browser console, and again, see how it has printed "Hello, world!".

Congratulations, you've written your first Zaplib program! 😄
