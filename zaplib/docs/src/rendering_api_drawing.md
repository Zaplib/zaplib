# Drawing

In a `draw()` function, [`Cx`](/target/doc/zaplib/struct.Cx.html) provides a few different functions to actually render to the screen.
 * Use [`add_instances`](/target/doc/zaplib/struct.Cx.html#method.add_instances) to render with a [`Shader`](/target/doc/zaplib/struct.Shader.html) and instance data. This will use the shader's `build_geom` as the rendered geometry.
 * Use [`add_mesh_instances`](/target/doc/zaplib/struct.Cx.html#method.add_mesh_instances) to render with a custom geometry, passing in a [`GpuGeometry`](/target/doc/zaplib/struct.GpuGeometry.html).
 * Use [`add_instances_with_scroll_sticky`](/target/doc/zaplib/struct.Cx.html#method.add_instances_with_scroll_sticky) to disable default scrolling behavior and keep items sticky on the screen. This is only relevant for 2D rendering that respects scrolling, such as UI components.

When calling one of these functions, under the hood we create a new `DrawCall` object, and nest it under the current `View`. However, a `DrawCall` is fairly expensive, so when possible we merge `DrawCall`s together. This is done when calling `cx.add_instances` multiple times in a row with the same shader. In that case we append the instance data to a single buffer, instead of creating multiple `DrawCall`s. In general we try to only do `DrawCall` batching when it doesn't alter any actual behavior.

### Shader groups

Sometimes it's useful be able to call `cx.add_instances` in a different order than you actually want to layer your draws. For example: when drawing a button, you might have a shader for the text and one for the background. In that case the background `DrawCall` should come first, followed by the text `DrawCall` which sits on top. But you might want to actually generate the text first, since that will determine the size of the button.

In such a scenario, it is of course possible to generate the `TextIns` objects first, then determine the button size, create the background `DrawCall`, and finally create the text `DrawCall` using the `TextIns` objects. But this often leads to ugly abstractions.

To solve for this, you can call [`cx.begin_shader_group`](/target/doc/zaplib/struct.Cx.html#method.begin_shader_group), which takes an array of `Shader`s in a certain order and will make sure the `DrawCall`s get ordered accordingly. You then close the group by calling `cx.end_shader_group`.

As a bonus, if you create multiple shader groups in a row with the same shaders, then we'll apply `DrawCall` batching on all the `DrawCall`s in those groups. This means that you can draw many buttons in a row, and still get batching on both the backgrounds and the texts. For big UIs this can make a substantial difference.
