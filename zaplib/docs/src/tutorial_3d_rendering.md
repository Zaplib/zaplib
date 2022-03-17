# Rendering 3D Meshes

This guide walks through Zaplib's rendering API. We'll go through a few steps:

1. Start with a JavaScript application, which extracts a geometry [from an STL file](https://en.wikipedia.org/wiki/File:Utah_teapot_(solid).stl) and renders it using ThreeJS and GLSL shaders.
2. Move our STL loading logic into Zaplib and communicate results using Web Workers.
3. Render using Zaplib.<!-- Potentially this step should be in its own guide -->

This guide assumes an understanding of JavaScript web development, basic 3D graphics, and writing GPU shaders using a shading language (such as GLSL/HLSL).

You can either follow this tutorial directly; creating the necessary files from scratch, or read the working incremental versions of each step, located in `zaplib/examples/tutorial_3d_rendering/`. To start from scratch, copy `zaplib/examples/tutorial_3d_rendering/step1` into a new directory at the top level of the `zaplib` repository called `tutorial_3d_rendering`.

## Step 1: Rendering a mesh in ThreeJS
This guide starts with a working 3D visualization in JavaScript, which renders an example using the popular [ThreeJS](https://threejs.org/) library. Let's take a look at our existing files.

Our `index.html` looks like the following:
```html
{{#include ../../examples/tutorial_3d_rendering/step1/index.html}}
```
In it, we define a top level full-page `div` with an id of `root`. We load `index.js` as well, reproduced below. Afterward, we'll go through its important pieces.

<details>
  <summary>index.js</summary>

```js
{{#include ../../examples/tutorial_3d_rendering/step1/index.js}}
```
</details>

This renders ThreeJS to the `root` div, which displays our 3D scene.

Let's focus on what is happening in the `init` function.

### First we have our ThreeJS boilerplating.
```js
{{#include ../../examples/tutorial_3d_rendering/step1/index.js:65:80}}
```
This does the following:
 - Defines a new 3D scene, rendering results to our supplied `div`.
 - Defines a camera using a perspective projection, a field of view of 40, and a near/far z-axis of 0.1 and 1000. These numbers are not specifically important, but they are the defaults in Zaplib's Viewport, which we'll see later.
 - Sets up OrbitControls, which lets us pan and zoom around our scene easily. OrbitControls by default lets us use the left mouse button to rotate the camera around the origin (0,0,0) while maintaining camera distance, and the right mouse button to pan around the scene freely.

### Defining geometry
```js
{{#include ../../examples/tutorial_3d_rendering/step1/index.js:82:84}}
```
This defines a ThreeJS InstancedMesh using a custom geometry and material. We supply an instance count of `3`, meaning that we will be rendering our model three times, with some custom properties per instance.

The geometry is loaded from a remote STL file using `loadSTLIntoGeometry`, let's take a look at that.
```js
{{#include ../../examples/tutorial_3d_rendering/step1/index.js:4:36}}
```

Without going line by line here, the function does the following:
  - Fetch a remote asset and load its result into an ArrayBuffer. We'll be rendering a [Utah teapot](https://en.wikipedia.org/wiki/Utah_teapot) - by default this will be available in `zaplib/examples/tutorial_3d_rendering/`.
  - Read through the buffer, extracting information for each triangle one by one. [See the binary STL spec here](https://en.wikipedia.org/wiki/STL_(file_format)#Binary_STL) for information about its structure. We load each vertex and its corresponding normal into Float32Arrays.
  - Define a new ThreeJS BufferGeometry and create new attributes to define its shapes.
    - The extracted position and normal data is loaded in as BufferAttributes.
    - We give each instance a y-axis offset, represented as floats, and load it as InstanceBufferAttribute.
    - We give each instance a color, represented as RGB values, and load it as an InstancedBufferAttribute.

Our mesh's material is specified using a ShaderMaterial, and aims to provide very basic lighting with a fixed point light.

The vertex shader saves our position and normal as varying parameters to be used in the fragment shader, and converting our position from world coordinates to screen coordinates. We apply our instance `offset` value to get a final position.
```glsl
{{#include ../../examples/tutorial_3d_rendering/step1/index.js:40:50}}
```
The fragment shader specifies a fixed light source and calculates pixel color by multiplying our instance color and  light intensity, using the dot product of the light direction and normal vector. We clamp light intensity between 0 and 1.
```glsl
{{#include ../../examples/tutorial_3d_rendering/step1/index.js:53:60}}
```

Great! Running the example, we see our 3D scene with rudimentary lighting and pan/zoom controls with the mouse. There is a delay between page loading and scene rendering, due to our STL extraction code.

## Step 2: STL extraction in Rust
WebAssembly and Rust are most useful for expensive operations, so let's offload STL extraction from JavaScript there and look at the tradeoffs. At a high level, this means:
 * performing STL extraction using a Web Worker and therefore parallel to our main thread
 * performing our network request for the STL file in Rust
 * communicating our result buffer back to JavaScript

As a reminder, a working example at the end of this step is available in `zaplib/examples/tutorial_3d_rendering/step2`.

### Instantiate a new Zaplib project
This structure is further explained in previous tutorials. We'll need:
 * a `Cargo.toml` file with the Zaplib dependency.
```toml
[package]
name = "tutorial_3d_rendering"
version = "0.0.1"
edition = "2018"

[dependencies]
zaplib = { path = "../zaplib/main" }
```
 * and a Zaplib entrypoint for WebAssembly, in `src/main.rs`.
```rust,noplayground
use zaplib::*;

fn call_rust(_name: String, _params: Vec<ZapParam>) -> Vec<ZapParam> {
    vec![]
}

register_call_rust!(call_rust);
```

### Port STL loading to Rust
Add a function to `src/main.rs` for STL loading. We can mirror the algorithm we have in JavaScript. Here's what that looks like:
```rust,noplayground
{{#include ../../examples/tutorial_3d_rendering/step2/src/main.rs:7:35}}
```
This code looks mostly the same; here are a few notable differences:
  * We make a web request and read to a file using Zaplib's `UniversalFile` API.
  * We use Zaplib's performant `byte_extract` module to read data. This must be imported by adding `use zaplib::byte_extract::{get_f32_le, get_u32_le};`. The module provides both little endian and big endian extraction functions for different primitive types.
  * We use the `into_param()` helper to convert Float32 vectors into params we can return to JavaScript.

We then integrate this to `call_rust`:
```rust,noplayground
{{#include ../../examples/tutorial_3d_rendering/step2/src/main.rs:37:43}}
```

To build, run:
```bash
cargo zaplib build -p tutorial_3d_rendering
```

### Calling from JS
To call this function from our JavaScript, let's add the Zaplib dependency to `index.html`. Add a line in the `<body>` section:
```html
{{#include ../../examples/tutorial_3d_rendering/step2/index.html:5}}
```

Then, modify `loadSTLIntoGeometry` to replace our JavaScript parsing code.
```js
{{#include ../../examples/tutorial_3d_rendering/step2/index.js:4}}
    await zaplib.initialize({ wasmModule: '/target/wasm32-unknown-unknown/debug/tutorial_3d_rendering.wasm' });
{{#include ../../examples/tutorial_3d_rendering/step2/index.js:6:13}}
```
A few key changes:
 * We call `zaplib.initialize` with the location of our built WebAssembly binary.
 * `zaplib.callRustSync` returns our vertices and normals already as Float32Arrays, which we can plug into ThreeJS.

Great! Now let's run the example in the browser. There should be no difference in the behavior and things will load as normal, without blocking our main browser thread.
<!-- Maybe there should be an actual observable difference here. -->

## Step 3 - Rendering in Zaplib
In addition to processing tasks, we can also render to the DOM directly from Rust using Zaplib. We can draw UI primitives as well as a full 3D Viewport, which will get output to a `canvas` element on our webpage.

For an introduction to basic rendering, take a look at [Tutorial: Hello World Canvas](./tutorial_hello_world_canvas.md). Just like in that tutorial, let's create a basic Zaplib application. Here is how our Rust code should look at this point:

```rust,noplayground
#[derive(Default)]
struct App {
    window: Window,
    pass: Pass,
    view: View,
}

impl App {
    fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    fn handle(&mut self, _cx: &mut Cx, _event: &mut Event) {}

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("0"));
        self.view.begin_view(cx, LayoutSize::FILL);

        cx.begin_padding_box(Padding::hv(50., 50.));
        TextIns::draw_walk(cx, "Hello, World!", &TextInsProps::default());
        cx.end_padding_box();

        self.view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(App);
```

Now we just need to connect the rendering with javascript page. To do so, remove our ThreeJS render, commenting out the entirety of `index.js` and replacing it with:
```js
zaplib.initialize({ wasmModule: '/target/wasm32-unknown-unknown/debug/tutorial_3d_rendering.wasm', defaultStyles: true });
```
Note the addition of `defaultStyles`, which will style our full-screen canvas correctly and add a loading indicator.

Rebuild the WebAssembly binary and refresh the screen. You should see a black background and a Hello World. Congratulations, we're rendering from Rust! ⚡️

### Rendering a 3D Viewport
Let's get back to our 3D example. One of the major advantages of Zaplib is the ability to use common structs for renderable data, instead of positional TypedArrays in JavaScript. In ThreeJS, we had to provide attributes as floats, but here we can be a bit more descriptive.

#### Generating geometries
Let's represent a vertex struct as the below and add it to `src/main.rs`.
```rust,noplayground
{{#include ../../examples/tutorial_3d_rendering/step3/src/main.rs:9:13}}
```
For each vertex of our shape, we represent each position and normal as a `Vec3`, which is a three-dimensional vector of floats. We have to add `#[repr(C)]` to indicate C struct alignment.

Let's also add an instance struct as the below.
```rust,noplayground
{{#include ../../examples/tutorial_3d_rendering/step3/src/main.rs:15:19}}
```
Like in JavaScript, we provide a Y-axis offset and color per instance. This data is fixed, so we can provide it as a static. Note how much more readable this is than linear buffers in JavaScript.
```rust,noplayground
{{#include ../../examples/tutorial_3d_rendering/step3/src/main.rs:21:25}}
```

Modify the `parse_stl` function now to generate a Zaplib geometry instead of float arrays. Let's take a look at the final function.
```rust,noplayground
{{#include ../../examples/tutorial_3d_rendering/step3/src/main.rs:62:106}}
```
Note:
 * Our vertex attributes are now represented by a `Vec<Vertex>` instead of multiple arrays.
 * We must generate a vector of `indices` to map vertices to triangles. Our approach here is naive, but this can be very useful for reducing memory costs when many vertices are duplicated.
 * Our resulting vertices and indices are eventually passed to `GpuGeometry::new`, which registers the geometry with the framework and makes it available on our GPU.

#### Generating geometry on startup
We now need a way to actually call `parse_stl` and save our geometry. Our `handle` function is the main entrypoint into the application lifecycle. One of our event types is called `Event::Construct`, called once after the framework has loaded. This sounds like a good place to load geometry. Write the `handle` function as follows.
```rust,noplayground
fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
    if let Event::Construct = event {
        self.geometry = Some(parse_stl(cx, "zaplib/examples/tutorial_3d_rendering/teapot.stl"));
        cx.request_draw();
    }
}
```
and add the geometry to `App`.
```rust,noplayground
#[derive(Default)]
struct App {
    window: Window,
    pass: Pass,
    main_view: View,
    geometry: Option<GpuGeometry>,
}
```
Note:
 * We pattern match on `event`, which is an enum of all possible event types.
 * `geometry` is saved as an `Option` type, because it will be `None` initially before loading.
 * We call `cx.request_draw` after this is done to tell our framework to draw. This function is the only way to force re-draws.

#### Defining the shader
We need a shader to represent how to render our geometry to screen, the same way we defined a `ShaderMaterial` in ThreeJS. Zaplib uses custom shader dialect, which looks similar to Rust code and is cross-platform compatible with web and native graphics frameworks. Define this shader above the `App` struct definition.
```rust,noplayground
{{#include ../../examples/tutorial_3d_rendering/step3/src/main.rs:27:51}}
```
Read the above carefully, and compare it to our previous JavaScript shader, reproduced below.
```js
{{#include ../../examples/tutorial_3d_rendering/step2/index.js:15:34}}
```
Some key differences:
 * Zaplib shaders take in both a default geometry and an array of shader fragments to concatenate. We pass in `None` since we are defining a custom geometry, and prepend `Cx::STD_SHADER` to get default shader properties.
 * Like in JS, we use `instance` parameters. The order here is very important and must match the alignment of the `Instance` struct, as we interpret it linearly.
 * We use the `geometry` parameter to deconstruct the values of our vertex attributes. The order here is similarly important.
 * Instance and geometry arameters are available to both fragment and vertex shaders, so we do not need to use `varying` variables to forward them.

#### Drawing a mesh
Now that we have both the geometry and shader defined, we can add our geometry to a Viewport3D. The Viewport, like many other UI widgets from Zaplib, is provided by the `zaplib_widget` crate. Add it as a dependency in `Cargo.toml`
```toml
zaplib_widget = { path = "../zaplib/widget" }
```
and import it at the top of `src/main.rs`.
```rust,noplayground
use zaplib_widget::*;
```

In our `draw` function, add the following between `begin_view` and `end_view`.
```rust,noplayground
{{#include ../../examples/tutorial_3d_rendering/step3/src/main.rs:125:137}}
```
In short, this checks if we have a loaded geometry and if so, draws a viewport with an instance of it. We define an `initial_camera_position` with the same coordinates as our ThreeJS sketch.

Add `viewport_3d` to the application struct
```rust,noplayground
{{#include ../../examples/tutorial_3d_rendering/step3/src/main.rs:53:60}}
```

Rebuild the application and refresh your browser. Whoa, we're now fully rendering 3D geometry in Rust!

Lastly, let's add camera controls like ThreeJS's OrbitControls. `Viewport3D` has this out of the box, but we need to make sure our event handler forwards events to it, so call `viewport_3d.handle` at the top of your `handle` function.
```rust,noplayground
{{#include ../../examples/tutorial_3d_rendering/step3/src/main.rs:113:120}}
```

Build and run the application. Pan and rotate with the mouse buttons, and enjoy your new WebAssembly rendered graphics!
