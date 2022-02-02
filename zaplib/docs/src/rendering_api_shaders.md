# Shaders

A [`Shader`](/target/doc/zaplib/struct.Shader.html) represents the program sent to the GPU to render each pixel on the surface of our geometry. It can be instantiated with a base geometry and must be provided with a shader written with our custom shading language.

A simple shader might look something like this:

```rust,noplayground
static SHADER: Shader = Shader {
    build_geom: Some(build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        code_fragment!(
            r#"
            geometry geom: vec2;
            instance color: vec4;

            fn vertex() -> vec4 {
                return vec4(geom.x, geom.y, 0., 1.);
            }

            fn pixel() -> vec4 {
                return color;
            }
            "#
        ),
    ],
    ..Shader::DEFAULT
};
```

Shaders are statically defined, and consist of two fields:
* `build_geom`: a function that produces a [`Geometry`](./rendering_api_overview_geometry.md). Can be omitted if you want to dynamically assign a geometry at draw time.
* `code_to_concatenate`: an array of [`CodeFragment`s](/target/doc/zaplib/struct.CodeFragment.html), that get concatenated in order. Define each fragment using the [`code_fragment!()`](/target/doc/zaplib/macro.code_fragment.html) macro (this keeps track of filenames and line numbers, for better error messages).

## Passing in data

A shader typically starts with a bunch of variable declarations. These declarations define the data that you pass into the shader, and has to exactly match the data types in Rust.

For example, to pass in instance data, you can define some `instance` variables in the shader:

```rust,noplayground
r#"
instance pos: vec2;
instance color: vec4;
"#
```

Which has to exactly match the corresponding "instance struct":

```rust,noplayground
#[repr(C)]
struct MyShaderIns {
    pos: Vec2,
    color: Vec4,
}
```

Note the use of `#[repr(C)]` to ensure that the data is properly laid out in memory.

When calling `cx.add_instances(&SHADER, &[MyShaderIns { pos, color }])`, we verify that the memory size of `MyShaderIns` matches that of the `instance` variables in the shader code.

This is how Rust types match with shader types:

| Rust | Shader |
|------|--------|
| f32  | float  |
| Vec2 | vec2   |
| Vec3 | vec3   |
| Vec4 | vec4   |
| Mat4 | mat4   |

Note that within a function there are [more types](/target/doc/zaplib/enum.Ty.html) you can use.

These are the types of variables you can declare:
* `geometry`: these have to match exactly the `vertex_attributes` fields in [`Geometry::new`](/target/doc/zaplib/struct.Geometry.html#method.new).
* `instance`: these have to match exactly the `data` fields in [`Cx::add_instances`](/target/doc/zaplib/struct.Cx.html#method.add_instances).
* `uniform`: these have to match exactly the `uniforms` fields in [`Area::write_user_uniforms`](/target/doc/zaplib/enum.Area.html#method.write_user_uniforms).
* `texture`: can only be of type `texture2D` and gets set using [`Cx::write_user_uniforms`](/target/doc/zaplib/enum.Area.html#method.write_user_uniforms).
* `varying`: doesn't get passed in from Rust, but can be used to pass data from `fn vertex()` to `fn pixel()`.

## Shader language

The shader language itself is modeled after Rust itself. You can use things like `fn`, `struct`, and so on. Two functions need to be defined for a shader to work:
* `fn vertex()` defines the vertex shader. This gets called for each vertex returned from `build_geom`. It returns `vec4(x, y, z, w)` where the values mean the following:
  * `x, y` — coordinates on the screen (from -1 to 1).
  * `z` — draw order (from 0 to 1). Draws with higher `z` will be on top.
  * `w` — normalization parameter. In 2d rendering this is simply set to 1.0.
* `fn pixel()` defines the pixel shader. It returns a color as `vec4(r, g, b, a)`.

See [Tutorial: Rendering 2D Shapes](./tutorial_2d_rendering.md) for more about the basics of drawing in a shader.

<table>
<thead><tr><td>Keyword</td><td>Description</td><td>Example</td></tr></thead>
<tr><td>const</td><td>Constant values</td><td><code>const PI: float = 3.141592653589793;</code></td></tr>
<tr><td>let</td><td>Variable (mutable)</td><td><code>let circle_size = 7. - stroke_width / 2.;</code></td></tr>
<tr><td>return</td><td>Return value</td><td><code>return 0.0;</code></td></tr>
<tr><td>if</td><td>Condition</td><td><pre><code>if errored > 0. {
    df.fill(error_color);
} else if loaded > 0. {
    df.fill(active_color);
} else {
    df.fill(inactive_color);
}</code></pre></td></tr>
<tr><td>#hex</td><td>Color</td><td><pre><code>return #ff0000;
return #f00;
return #f;</code></pre></td></tr>
<tr><td>fn</td><td>Function definition</td><td><pre><code>fn pixel() -> vec4 {
    return #f00;
}</code></pre></td></tr>
<tr><td>struct</td><td>Structure definition</td><td><pre><code>struct Df {
    pos: vec2,
    result: vec4,
}</code></pre></td></tr>
<tr><td>impl</td><td>Structure implementation</td><td><pre><code>impl Df {
    fn clear(inout self, color: vec4) {
        self.write_color(color, 1.0);
    }
}</code></pre></td></tr>
<tr><td>for</td><td>Range loop</td><td><pre><code>for i from 0 to 20 step 3 {
    if float(i) >= depth {
        break;
    }
}</code></pre></td></tr>
<tr><td>?</td><td>Ternary operator</td><td><code>let pos = is_left ? start : end;</code></td></tr>
</table>

The following built-in functions are available: [abs](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/abs.xhtml), [acos](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/acos.xhtml), [acos](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/acos.xhtml), [all](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/all.xhtml), [any](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/any.xhtml), [asin](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/asin.xhtml), [atan](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/atan.xhtml), [ceil](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/ceil.xhtml), [clamp](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/clamp.xhtml), [cos](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/cos.xhtml), [cross](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/cross.xhtml), [degrees](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/degrees.xhtml), [dFdx](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/dFdx.xhtml), [dFdy](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/dFdy.xhtml), [distance](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/distance.xhtml), [dot](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/dot.xhtml), [equal](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/equal.xhtml), [exp](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/exp.xhtml), [exp2](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/exp2.xhtml), [faceforward](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/faceforward.xhtml), [floor](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/floor.xhtml), [fract](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/fract.xhtml), [greaterThan](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/greaterThan.xhtml), [greaterThanEqual](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/greaterThanEqual.xhtml), [inversesqrt](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/inversesqrt.xhtml), [inverse](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/inverse.xhtml), [length](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/length.xhtml), [lessThan](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/lessThan.xhtml), [lessThanEqual](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/lessThanEqual.xhtml), [log](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/log.xhtml), [log2](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/log2.xhtml), [matrixCompMult](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/matrixCompMult.xhtml), [max](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/max.xhtml), [min](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/min.xhtml), [mix](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/mix.xhtml), [mod](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/mod.xhtml), [normalize](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/normalize.xhtml), [not](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/not.xhtml), [notEqual](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/notEqual.xhtml), [pow](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/pow.xhtml), [radians](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/radians.xhtml), [reflect](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/reflect.xhtml), [refract](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/refract.xhtml), [sample2d](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/sample2d.xhtml), [sign](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/sign.xhtml), [sin](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/sin.xhtml), [smoothstep](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/smoothstep.xhtml), [sqrt](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/sqrt.xhtml), [step](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/step.xhtml), [tan](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/tan.xhtml), [transpose](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/transpose.xhtml).

[Swizzling](https://www.khronos.org/opengl/wiki/Data_Type_(GLSL)#Swizzling) is also supported, for both `xyzw` and `rgba`. So you can do things like `let plane: vec2 = point.xy` or `let opaque: vec3 = color.rgba`.

## STD_SHADER

Zaplib provides [STD_SHADER](/target/doc/zaplib/struct.Cx.html#associatedconstant.STD_SHADER), a collection of common functions that are useful when writing shaders. For a complete run down on the available functions, it's best to directly look at the source, but we'll discuss some highlights.

### 3D space transforms
These values are useful when working in 3D space, translating an object's scene coordinates into pixel locations on the screen.

| Name | Type | Description |
|-|-|-|
| camera_projection | mat4 |  Camera projection matrix - see [3D projection](https://en.wikipedia.org/wiki/3D_projection). |
| camera_view | mat4 | View matrix - see [Camera matrix](https://en.wikipedia.org/wiki/Camera_matrix). |
| inv_camera_rot | mat4 | The inverse rotation matrix for a camera. Useful for working with billboards. |

As a quick example, a basic vertex shader to convert from object to screen coordinates is:
```rust,noplayground
fn vertex() -> vec4 {
    return camera_projection * camera_view * vec4(geom_position, 1.);
}
```

These values get set by a combination of [`Pass::set_matrix_mode`](/target/doc/zaplib/struct.Pass.html#method.set_matrix_mode) and the actual computed dimensions of a [`Pass`](/target/doc/zaplib/struct.Pass.html). See e.g. the [`Viewport3D`](/target/doc/zaplib_components/struct.Viewport3D.html) component.

### Rendering helpers
| Name | Type | Description |
|-|-|-|
| dpi_factor | float |  More commonly known as the "device pixel ratio"; represents the ratio of the resolution in physical pixels to the resolution in GPU pixels for the current display device. |
| dpi_dilate | float | Some amount by which to thicken lines, depending on the `dpi_factor` |
| draw_clip | vec4 | [Clip region](https://en.wikipedia.org/wiki/Clipping_(computer_graphics)) for rendering, represented as (x1,y1,x2,y2). |
| draw_scroll | vec2 | The total 2D scroll offset, including all its parents. This is usually only relevant for 2D UI rendering. |
| draw_local_scroll | vec2 | The 2D scroll offset excluding parents. This is usually only relevant for 2D UI rendering. |
| draw_zbias | float | A small increment that you can add to the z-axis of your vertices, which is based on the position of the draw call in the draw tree. |

### Math
| Name | Type | Description |
|-|-|-|
| Math::rotate_2d | (v: vec2, a: float) -> vec2 | Rotate vector `v` by radians `a` |

### Colors
| Name | Type | Description |
|-|-|-|
| hsv2rgb | (c: vec4) -> vec4  | Convert color `c` from [HSV representation](https://en.wikipedia.org/wiki/HSL_and_HSV) to [RGB representation](https://en.wikipedia.org/wiki/RGB_color_model) |
| rgb2hsv | (c: vec4) -> vec4  | Convert color `c` from [RGB representation](https://en.wikipedia.org/wiki/RGB_color_model) to [HSV representation](https://en.wikipedia.org/wiki/HSL_and_HSV) |

### Useful constants
| Name | Type | Description |
|-|-|-|
| PI | float | [Pi (π)](https://en.wikipedia.org/wiki/Pi) |
| E | float | [e](https://en.wikipedia.org/wiki/E_(mathematical_constant)) |
| LN2 | float | ln(2) - The natural log of 2 |
| LN10 | float | ln(10) - The natural log of 10 |
| LOG2E | float | log2(e) - Base-2 log of e |
| LOG10E | float | log2(e) - Base-10 log of e |
| SQRT1_2 | float | sqrt(1/2) - Square root of 1/2 |
| TORAD | float | Conversion factor of degrees to radians. Equivalent to PI/180. |
| GOLDEN | float | [Golden ratio](https://en.wikipedia.org/wiki/Golden_ratio) |

### Distance fields
Zaplib contains many functions for [Signed Distance Fields (SDFs)](https://jasmcole.com/2019/10/03/signed-distance-fields/) under the `Df` namespace. SDFs are a comprehensive way to define flexible shapes on the GPU. While applicable in 2D and 3D contexts, Zaplib uses this only for 2D rendering.

To create a distance field, use either:

| Name | Type | Description |
|-|-|-|
| Df::viewport | (pos: vec2) -> Df | Creates a distance field with the current position |
| Df::viewport_px | (pos: vec2) -> Df | Creates a distance field with the current position, factoring in `dpi_factor` |

The following methods are available on the instantiated `Df` struct.

| Name | Type | Description |
|-|-|-|
| df.add_field | (field: float) -> void | Adds a new field value to the current distance field |
| df.add_clip | (d: float) -> void | Adds a clip mask to the current distance field |
| df.antialias | (p: vec2) -> float | Distance-based antialiasing |
| df.translate | (offset: vec2) -> vec2 | Translate a specified offset |
| df.rotate | (a: float, pivot: vec2) -> void | Rotate by `a` radians around `pivot` |
| df.scale | (f: float, pivot: vec2) -> void | Uniformly scale by factor `f` around `pivot` |
| df.clear | (src: vec4)  -> void | Sets clear color. Useful for specifying background colors before rendering a path. |
| df.new_path | () -> void | Clears path in current distance field. |
| df.fill | (color: vec4) -> vec4 | Fills the current path with `color`. |
| df.stroke | (color: vec4, width: float) -> vec4 | Strokes the current path with `color` with a pixel width of `width`. |
| df.glow | (color: vec4, width: float) -> vec4 | Updates the current path by summing colors in `width` with the provided one. |
| df.union | () -> void | Set field to the union of the current and previous field. |
| df.intersect | () -> void | Set field to the intersection of the current and previous field. |
| df.subtract | () -> void | Subtract current field from previous. |
| df.blend | (k: float) -> void | Interpolate current field and previous with factor `k`. |
| df.circle | (p: vec2, r: float) -> void | Render a circle at `p` with radius `r`. |
| df.arc | (p: vec2, r: float, angle_start: float, angle_end: float) -> void | Render an arc at `p` with radius `r` between angles `angle_start` and `angle_end`. |
| df.rect | (p: vec2, d: vec2) -> void | Render a rectangle at `p` with dimensions `d`. |
| df.box | (p: vec2, d: vec2, r: float) -> void | Render a box with rounded corners at `p` with dimensions `d`. Use `r` to indicate the corner radius - if `r` is less than 1, render a basic rectangle. If `r` is bigger than `min(w, h)`, the result will be a circle. |
| df.triangle | (p0: vec2, p1: vec2, p2: vec2) -> void | Render a triangle between points  `p0`, `p1`, `p2`. |
| df.hexagon | (p: vec2, r: float) -> void | Render a hexagon at p with side length `r`. |
| df.move_to | (p: vec2) -> void | Move to `p` in current path, not drawing from current position. |
| df.line_to | (p: vec2) -> void | Render a line to `p` from current position. |
| df.close_path | () -> void | End the current field by rendering a line back to the start point. |

