# Tutorial: Hello World Console

Let's write the most basic application: printing "Hello, world!" to the console. You can follow along in [`zaplib/examples/tutorial_hello_world_console`](https://github.com/Zaplib/zaplib/tree/main/zaplib/examples/tutorial_hello_world_console).

## 1. `Cargo.toml`

The `Cargo.toml` file is where you configure your project much like you would in a `package.json`. This one simply imports Zaplib locally. If you use Zaplib outside of this repo, you'd need to [specify a version](./versioning.md).

```toml
{{#include ../../examples/tutorial_hello_world_console/Cargo.toml}}
```
## 2. `src/main.rs`

The `src/main.rs` file is the entrypoint for your application.

```rust,noplayground
{{#include ../../examples/tutorial_hello_world_console/src/main.rs:main}}
```

Let's break it down a bit. (For now, try to ignore the [`&mut` type annotations](./resources.html#ownership--borrowing).) `App` is a `struct` that implements three methods:

1. `new` returns an initialized `App` `struct`.
2. `handle` is the entrypoint into Zaplib's event handling system. We will go in depth on various event types in a different tutorial. For now, we'll put our `log!()` call in the the `Construct` event.
3. `draw` is called when requesting a draw. This will control what gets shown on the application window, which we don't use yet.

The call to `main_app!()` tells Zaplib to use the `App` struct for all its eventing and rendering.

## 3. Run the app natively

In your shell:

```
cargo run -p tutorial_hello_world_console
```

Hurray! It prints `Hello, world!` 🥳 

Notice how this program currently never exits on its own. That behavior is similar to the web version, where the program doesn't exit until the browser window is closed. In our case here we don't have a native window yet, so terminate the program using CTRL+C.

## 3. Compile to WebAssembly

```
cargo zaplib build -p tutorial_hello_world_console
```

## 4. `index.html`

The ```index.html``` file simply imports the Zaplib runtime, and initializes it by pointing at the compiled WASM file for this example. If you use Zaplib outside of this repo, you'd likely use npm or yarn to install and import the Zaplib runtime.

```html
{{#include ../../examples/tutorial_hello_world_console/index.html}}
```

## 5. Run the server

In your shell:

```
cargo zaplib serve
```
## 5. Open the app in Chrome

<a href="http://localhost:3000/zaplib/examples/tutorial_hello_world_console" target="_blank">http://localhost:3000/zaplib/examples/tutorial_hello_world_console</a>

Open the browser console, and see how it has printed "Hello, world!".

Congratulations, you've written your first Zaplib program! 😄
