# Tutorial: Hello Thread, Hello File

This tutorial creates a simple app that accepts files via drag-and-drop, and prints their contents to the console. 

We build on the [Hello World](./tutorial_hello_world_console.md) tutorial by modifying it in pieces. You can follow along by modifying those files, or via the completed [`zaplib/examples/tutorial_hello_thread`](https://github.com/Zaplib/zaplib/tree/main/zaplib/examples/tutorial_hello_thread) example.

## Universal interfaces

Many standard Rust functions don't work in WebAssembly out-of-the-box, such as threading or reading files. Zaplib provides a number of universal interfaces that work both natively and in WebAssembly. This tutorial demonstrates two cross-platform Zaplib abstractions: 

1. `universal_thread`
2. `UniversalFile`

## 1. A Simple Loop

Let's start by logging to the console multiple times:

```rust,noplayground
for i in 0..3 {
    log!("Hello, world! {i}");
}
```

You can run these examples either natively or in the browser. How to run a Zaplib app is covered in [Getting Started](./getting_started.html#examples) and the [Hello World](./tutorial_hello_world_console.md) tutorial.

 
The output is as expected:

```
zaplib/examples/tutorial_hello_world_console/src/main.rs:22 - Hello, world! 0
zaplib/examples/tutorial_hello_world_console/src/main.rs:22 - Hello, world! 1
zaplib/examples/tutorial_hello_world_console/src/main.rs:22 - Hello, world! 2
```

## 2. `universal_thread`

Let's put each `log!` in a thread:

```rust,noplayground
for i in 0..3 {
    universal_thread::spawn(move || {
        log!("Hello, world! {i}");
    });
}
```

Now the ordering of the print statements is non-deterministic:

```
zaplib/examples/tutorial_hello_world_console/src/main.rs:22 - Hello, world! 2
zaplib/examples/tutorial_hello_world_console/src/main.rs:22 - Hello, world! 0
zaplib/examples/tutorial_hello_world_console/src/main.rs:22 - Hello, world! 1
```

Zaplib's `universal_thread` was designed to be a drop-in replacement for Rust's [std::thread](https://doc.rust-lang.org/std/thread/) with added support for WebAssembly. Natively it uses `std::thread`. In WebAssembly it uses Web Workers. 

If you're familiar with [Web Workers](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers), you'll notice how much easier Zaplib makes working with threads! The equivalent JavaScript using Web Workers and `postMessage` would be significantly more verbose, across multiple files. While threading is fairly advanced Rust, we think it is ultimately easier than threading in JavaScript.

## 3. `UniversalFile`

Let's read some files! 

In Rust, you'd normally use [`std::file::File`](https://doc.rust-lang.org/std/fs/struct.File.html), but it is not available in WebAssembly, so we use `UniversalFile`. Natively it uses `std::file::File`. In WebAssembly it makes an HTTP request for the file. The following example reads the project's `Cargo.toml` file. It works both natively by reading the file locally, like normal. It works in WebAssembly if your file server also serves your `Cargo.toml` file.

First, add the following import to the top of your file:

```rust,noplayground
use std::io::Read;
```

Next, let's write replace the loop of logging threads with:

```rust,noplayground 
let path = "zaplib/examples/tutorial_hello_world_console/Cargo.toml";
let mut file = UniversalFile::open(path).unwrap(); // open the file
let mut contents = String::new();                  // a string to hold the contents
file.read_to_string(&mut contents).unwrap();       // read the file into the string
log!("Contents of Cargo.toml: {contents}");        // log the file's contents
```

Note that this is a synchronous API: `read_to_string` will block until the whole file is read. JavaScript typically solves this via `Promise`, `async`, and `await`. While Rust does have `async` capabilities, we find that it quickly land you in "async hell" with Rust's borrow checker. In Zaplib — and low-level programming in general — we instead use threads to do things in an unblocking way:

```rust,noplayground
universal_thread::spawn(move || {
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

This code is from [`zaplib/examples/tutorial_hello_thread`](https://github.com/Zaplib/zaplib/tree/main/zaplib/examples/tutorial_hello_thread), so you can just run `cargo run -p tutorial_hello_thread` to run it natively.

Run this either natively or in WebAssembly, and then drag in a small text file onto the screen. It should print the contents to the console. Since we did the file reading in a thread, it won't block any other code; though in this example it's hard to tell the difference both because there's no other compute happening and because Zaplib runs your Rust code in a Web Worker anyway. 😉

If you're actually going to do file reading, be sure to read up on the [`std::file::File`](https://doc.rust-lang.org/std/fs/struct.File.html) documentation, since the advice there still applies (e.g. it's often a good idea to wrap things in a `BufReader`).
