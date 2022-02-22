# Tutorial: Hello World Console

Let's write the most basic application: printing "Hello, world!" to the console.

You can follow along with the existing `tutorial_hello_world_console`. First, we create our `Cargo.toml`:

```toml
{{#include ../../examples/tutorial_hello_world_console/Cargo.toml}}
```

Now, we create `src/main.rs`:

```rust,noplayground
{{#include ../../examples/tutorial_hello_world_console/src/main.rs:main}}
```


(For now, try to ignore the [`&mut` type annotations](./resources.html#ownership--borrowing).) 

Let's break it down a bit. `App` is a `struct` that implements three methods:

- `new` returns an initialized `App` `struct`.
- `handle` is the entrypoint into Zaplib's event handling system. We will go in depth on various event types in a different tutorial. For now, we'll put our `log!()` call in the the `Construct` event.
- `draw` is called when requesting a draw. This will control what gets shown on the application window, which we don't use yet.

The call to `main_app!()` tells Zaplib to use the `App` struct for all its eventing and rendering.

This is already enough to run the native version: `cargo run -p tutorial_hello_world_console`. Hurray! It prints "Hello, world!".

Notice how this program currently never exits on its own. That behavior is similar to the web version, where the program doesn't exit until the browser window is closed. In our case here we don't have a native window yet, so terminate the program using CTRL+C.

### WebAssembly

1. Add an `index.html`:

```html
{{#include ../../examples/tutorial_hello_world_console/index.html}}
```

2. Compile to WebAssembly: 

```
cargo zaplib build -p tutorial_hello_world_console
```

3. Build the Zaplib runtime (normally imported as an npm package):

```
cd zaplib/web && yarn && yarn build
```

4. Run the server:

```
cargo zaplib serve
```

5. Navigate to [http://localhost:5000/zaplib/examples/tutorial_hello_world_console](http://localhost:5000/zaplib/examples/tutorial_hello_world_console), open the browser console, and see it printed "Hello, world!".

Congratulations, you've written your first Zaplib program! 😄
