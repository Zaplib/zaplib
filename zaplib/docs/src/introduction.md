# Introduction

**Zaplib** is an open-source library for speeding up web applications using Rust and WebAssembly. It lets you write high-performance code in Rust, alongside your existing JavaScript code.

The goal of Zaplib is to make it easy to build performance-intensive applications in the browser. While it is possible to make JavaScript run fast, over time it may become hard to manage lots of optimizations. In Rust you tend to need way fewer optimizations to get high levels of performance, so you can focus on actually building stuff.

Zaplib is designed to be **incrementally adoptable**. Start by porting over a single function you know is slow. Then port over an entire UI component, leaving the rest of your app alone. Over time, you could port your entire codebase over to Rust, or you might keep JavaScript and Rust code side-by-side.

Zaplib is in alpha, but it's rapidly improving. If you want to use this library in production, please say hi in our [Slack](/slack.html), so we can work with you on the integration. Don't be shy — please reach out if you run into any issues at all 😄

## Demo

The following demo (including the text!) is [fully rendered within Zaplib](https://github.com/Zaplib/zaplib/blob/main/zaplib/examples/example_lightning/src/main.rs): using Rust, in a Web Worker, and using our custom 2d rendering engine. We render the Zaplib lightning bolt logo, with draggable control points, and a live editable shader (in a fully Rust-based editor originally built by the <a href="https://github.com/makepad/makepad">Makepad</a> folks). Try changing `LINES` or `LINE_BASE_LENGTH`. It's best viewed on desktop in a modern browser. For more demos, check out [demos](./demos.md).

<div style="height: 600px"><iframe src="/example_lightning.html?release" style="position: absolute; left: 50%; transform: translateX(-50%); width: 100%; max-width: 1000px; height: 600px; border: none;"></iframe></div>

## Structure

Zaplib roughly consists of these parts:

1. **Standard Library** - logging, threading, HTTP, reading files, etc.
2. **JS bridge** - communicating data between JS and Rust.
3. **Rendering** - low-level GPU-based 2d and 3d rendering APIs, and eventing.
4. **UI** - components, layout engine, animation.

Current development is focused on 1 - 3. In the future we aim to support building entire applications fully within Zaplib.

## Build Targets

Zaplib supports the following build targets:

1. **WebAssembly / WebGL** - Tested on modern Chrome. Known issues in Firefox, Edge, Safari.
2. **Mac OS X / Metal** - Tested on 11.6 Big Sur.
3. **Linux / OpenGL** - Not well supported. Some APIs missing, but should run.
4. **Windows / DirectX 11** - Currently broken... sorry!
5. [**Zapium**](./zapium.html) - Zaplib's equivalent of Electron. Highly experimental.

Currently our main focus is Web Assembly / WebGL support, and native targets are  mostly used for a faster development cycle. (Compiling Rust to native is faster than to WebAssembly.)
## Development

Zaplib is open source, with the code hosted on [Github](https://github.com/Zaplib/zaplib). Communication happens on [Slack](/slack.html).

The open source core team consists of:

<div style="margin: 0; display: flex; flex-wrap: wrap; vertical-align: top">
        <div style="max-width: 150px; padding: 3px 20px; border: 1px var(--table-border-color) solid;"><a href="https://github.com/janpaul123"><img style="width: 150px; max-width: 150px" src="./img/jp.jpg"><br>JP Posma (Zaplib)</a></div>
        <div style="max-width: 150px; padding: 3px 20px; border: 1px var(--table-border-color) solid;"><a href="https://github.com/stevekrouse"><img style="width: 150px; max-width: 150px" src="./img/steve.jpg"><br>Steve Krouse (Zaplib)</a></div>
        <div style="max-width: 150px; padding: 3px 20px; border: 1px var(--table-border-color) solid;"><a href="https://github.com/disambiguator"><img style="width: 150px; max-width: 150px" src="./img/paras.jpg"><br>Paras Sanghavi (Cruise)</a></div>
        <div style="max-width: 150px; padding: 3px 20px; border: 1px var(--table-border-color) solid;"><a href="https://github.com/hhsaez"><img style="width: 150px; max-width: 150px" src="./img/hernan.png"><br>Hernan Saez (Cruise)</a></div>
        <div style="max-width: 150px; padding: 3px 20px; border: 1px var(--table-border-color) solid;"><a href="https://github.com/pankdm"><img style="width: 150px; max-width: 150px" src="./img/dmitry.jpg"><br>Dmitry Panin (Cruise)</a></div>
</div>

Also a big shoutout to the <a href="https://github.com/makepad/makepad">Makepad</a> folks, whose open source framework we originally forked and with whom we've had a fruitful collaboration ever since.

## License

Zaplib is distributed under the terms of both the MIT license and the Apache License (version 2.0).

See `LICENSE-APACHE` and `LICENSE-MIT` in the repo root for details. Third party license notices are available in `LICENSES-THIRD-PARTY`.

We're currently exploring how to make a sustainable company around Zaplib. We hope and expect to keep the vast majority of code open source, but there is a possibility that some parts of this repo (such as [Zapium](./zapium.md) for example) move to a source-available but more restrictive license in the future. The trust of the open source community is our biggest asset, so we'd always be very careful and communicative about such decisions.
