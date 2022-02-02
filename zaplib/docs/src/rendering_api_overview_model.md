# Rendering model

The entrypoint of an application is typically the [`main_app!()`](/target/doc/zaplib/macro.main_app.html) macro. A minimal example looks like this:

```rust,noplayground
#[derive(Default)]
struct App {}

impl App {
    fn new(cx: &mut Cx) -> Self {
        Self::default()
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {}

    fn draw(&mut self, cx: &mut Cx) {}
}

main_app!(App);
```

Let's break it down a bit. The app must be a `struct` that implement three methods:
* `fn new()` — Returns an initialized struct and any initial state we add. Gets only called once.
* `fn handle()` — An entrypoint into Zaplib's event handling system.
* `fn draw()` — Called when a draw has been requested, e.g. on startup, during resizing, or on [`cx.request_draw()`](/target/doc/zaplib/struct.Cx.html#method.request_draw).

## Draw tree

Under the hood, the core data structure is a **"draw tree"**. This contains all information that we need to tell the GPU what to draw on the screen. There are two phases of rendering:
* **Drawing**: this is the process of generating a new draw tree. It might share data with the previous draw tree for caching purposes, but conceptually it's useful to think of it as producing a new draw tree.
* **Painting**: this is the process of informing the GPU of the new data in the draw tree. Painting always happens after drawing, but might also be done independently.

Here is how the two main top-level functions interact with these two phases:
* `fn handle()`
  * Called when an event is fired, such as a mouse movement.
  * Events do not trigger drawing or painting, but a redraw can be requested using [`cx.request_draw()`](/target/doc/zaplib/struct.Cx.html#method.request_draw). This will cause `fn draw()` to get called (once `fn handle()` has finished).
    * You'd typically call this whenever you update application state.
  * It is possible to directly access the draw tree here, e.g. by calling functions on [`Area`](/target/doc/zaplib/enum.Area.html) (which is a pointer into the draw tree).
  * When modifying the draw tree in place (e.g. with [`Area::get_slice_mut`](/target/doc/zaplib/enum.Area.html#method.get_slice_mut)), the modified draw tree gets painted afterwards.
    * Be careful with this, since your changes to the draw tree will get blown away the next we do drawing. Be sure to keep a single source of truth in both cases.
    * This can be useful for cheap, local modifications, like animations.
  * See [Events](./rendering_api_events_overview.md) for more details on handling events.
* `fn draw()`
  * Gets called when a draw is requested, either internally by the framework or by [`cx.request_draw()`](/target/doc/zaplib/struct.Cx.html#method.request_draw).
  * At the start, the entire draw tree is cleared out, except for some caching information.
  * Within this function, you make API calls to rebuild the draw tree again.
  * Afterwards, painting always happens.

The draw tree itself is a data structure that contains the following information:
* Shaders: a list of `Shader` objects, which are programs that run on the GPU.
* Geometries: a list of `GpuGeometry` objects, which are sets of vertices (points) that together form triangles, that are stored on the GPU.
* Windows: a list of `Window` objects, representing actual windows on the desktop. On WebAssembly there is only ever one window.
* Passes: a list of `Pass` objects, representing a render target. Comparable to `<canvas>` on the web. Each `Window` has one associated `Pass`, but you can also use `Pass`es to render to `Texture`s.
* Views: a list of `View` objects, which is mostly used as a scroll container, but is currently also required when you're not doing any scrolling. Each `Pass` has one main `View`. `View`s can also be nested.
* DrawCalls: `DrawCall` objects, which are instructions to draw something on the GPU, given a `Shader`, a `GpuGeometry`, a `View`, and a buffer of GPU instance data.
* Textures: `Texture` objects, which are buffers that are held on the GPU. You can write to them using a `Pass`, or read/modify them directly.

There is somewhat of a tree structure to the draw tree. Here is an example:
* `Window` (in WebAssembly there is only one window)
  * `Pass` (each `Window` has one main `Pass`, but `Pass` can also be created stand-alone)
    * `View` (each `Pass` has one main `View`)
      * `DrawCall` (points to a `Shader`, optionally `GpuGeometry`, and holds an instance data buffer)
      * `DrawCall`
      * `View` (`Views` can be nested arbitrarily deep, mostly when creating scroll containers)
        * `DrawCall`
        * `DrawCall`
      * `DrawCall`
      * `View`
        * `DrawCall`
* `Shader` (mostly separate; gets referred to from `DrawCall`)
* `Shader`
* `Shader`
* `GpuGeometry` (mostly separate; gets referred to from `Shader` or `DrawCall`)
* `GpuGeometry`
* `Texture` (can be read by a `DrawCall`, written to by a `Pass`, or read/written by Rust)
* `Texture`

Since at a minimum we need a `Window`, a `Pass`, and a `View`, there is a bit of boilerplate to get started with rendering. See [Tutorial: Hello World Canvas](./tutorial_hello_world_canvas.md) for an example.

## Painting

When painting, we traverse the draw tree down, creating commands for the GPU in the process. Typically it looks something like this:
1. Compiling shaders.
2. Computing which `Pass`es should be painted. For example, if a pass A renders a texture that is produced by pass B, and pass B has changed, then both passes will be painted. Under the hood we keep a dependency graph to figure this out.
3. For each `Pass`, render the main `View`. Rendering a `View` is a recursive process. We start off without any scrolling offsets, and no `zbias`. Then, we draw the children in order:
    * For each `DrawCall`, set the total scroll offset, clipping region, and `zbias` that we have accumulated so far. Then, queue up a paint command. When done, increment `zbias` by a small amount.
    * For each nested `View`, read out the local scroll position, add that to the accumulated total, and then recursively paint that `View`.
