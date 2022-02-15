# Tutorial: Hello Thread, Hello File

This tutorial builds on top of the previous [Hello World](./tutorial_hello_world_console.md) tutorial, by modifying it. Let's do some more stuff.

First, let's replace our "Hello, world!" logging, with spawning some threads:

```rust,noplayground
for i in 0..3 {
    universal_thread::spawn(move || {
        log!("Hello, world! {i}");
    });
}
```

`universal_thread` is our abstraction that works just like Rust's [std::thread](https://doc.rust-lang.org/std/thread/), but with added support for WebAssembly.

When running this (either natively or in WebAssembly), you will see something like:

```
zaplib/examples/tutorial_hello_world_console/src/main.rs:22 - Hello, world! 2
zaplib/examples/tutorial_hello_world_console/src/main.rs:22 - Hello, world! 0
zaplib/examples/tutorial_hello_world_console/src/main.rs:22 - Hello, world! 1
```

If you run it multiple times you'll see different orderings, since it's not deterministic which thread prints first.

Notice how relatively easy it was to spawn some threads, and transfer data into them (`i`), compared with using Web Workers and `postMessage`! Threading is still fairly advanced Rust, but in our experience, once you've gotten used to it, it ends up quite a bit easier to work with than threading in JavaScript.

## Reading files

Let's read some files! In Rust, you would normally use the [`std::file::File`](https://doc.rust-lang.org/std/fs/struct.File.html) object, but again, that is not available in WebAssembly. So instead, we use our `UniversalFile` abstraction. We read and print our `Cargo.toml` file:

```rust,noplayground
// Top of the file:
use std::io::Read;

// Replace the logging code with:
let path = "zaplib/examples/tutorial_hello_world_console/Cargo.toml";
let mut file = UniversalFile::open(path).unwrap();
let mut contents = String::new();
file.read_to_string(&mut contents).unwrap();
log!("Contents of Cargo.toml: {contents}");
```

This should now print the contents of Cargo.toml, both natively and in WebAssembly.

Note that this is a synchronous API, so it will block further execution. JavaScript typically solves this by using `Promise`s, potentially combined with `async` and `await`. In Rust — and native programming in general — we can solve this by instead putting our synchronous code in a thread:

```rust,noplayground
universal_thread::spawn(|| {
    let path = "zaplib/examples/tutorial_hello_world_console/Cargo.toml";
    let mut file = UniversalFile::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    log!("Contents of Cargo.toml: {contents}");
});
```

Since we're using a standard API interface, this code will work with any library that accepts a [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html) object, as opposed to WebAssembly libraries that expose more exotic asynchronous APIs.

## Drag & drop files

Now let's put it all together. This might be a bit overwhelming all at once, but it gives you a glimpse into how various APIs work, such as drawing, event handling, threading, and file reading.

```rust,noplayground
{{#include ../../examples/tutorial_hello_thread/src/main.rs}}
```

This code is also in the `tutorial_hello_thread` example, so you can just run `cargo run -p tutorial_hello_thread`.

Run this either natively or in WebAssembly, and then drag in a small text file. It should print the contents to the console. Since we did the file reading in a thread, it won't block any other code; though in this example it's hard to tell the difference. 😉

If you're actually going to do file reading, be sure to read up on the [`std::file::File`](https://doc.rust-lang.org/std/fs/struct.File.html) documentation, since the advice there still applies (e.g. it's often a good idea to wrap things in a `BufReader`).
