# Geometry

Geometries are sets of vertices (points) that together form triangles. It's the most basic rendering primitive in Zaplib. With them, we can create everything from interactive UI elements to 3D meshes.

A [`Geometry`](/target/doc/zaplib/struct.Geometry.html) describes the shape of a renderable item, represented as triangles. Create a `Geometry` using [`Geometry::new`](/target/doc/zaplib/struct.Geometry.html#method.new), which takes in both vertex attributes and indices to map vertices to triangle faces.

For example, take a look at our internal representation of [`QuadIns`](/target/doc/zaplib/struct.QuadIns.html). To represent the `Quad` shape, consider two right triangles both sharing a hypotenuse to form a square.
```rust,noplayground
{{#include ../../main/src/quad_ins.rs:build_geom}}
```

### GpuGeometry
A [`GpuGeometry`](/target/doc/zaplib/struct.GpuGeometry.html) is used to register a [`Geometry`](/target/doc/zaplib/struct.Geometry.html) with our application context. It is called via `GpuGeometry::new(cx, geometry)`. Under the hood, this is reference counted and can be cheaply cloned to add a new reference to the same geometry. When all references are dropped, the buffer will get reused in the next call to `GpuGeometry::new`.

### Usage

You can statically assign a `Geometry` to a `Shader`, by passing in a [`build_geom`](/target/doc/zaplib/struct.Shader.html#structfield.build_geom) when creating a `Shader`. To render such a shader, use [`add_instances`](/target/doc/zaplib/struct.Cx.html#method.add_instances).

It's also possible to omit a `build_geom` when creating a `Shader`, and instead dynamically assign it a `GpuGeometry` when drawing. In that case, use [`add_mesh_instances`](/target/doc/zaplib/struct.Cx.html#method.add_mesh_instances).

See [Drawing](./rendering_api_drawing.md) for more information on different APIs for drawing.
