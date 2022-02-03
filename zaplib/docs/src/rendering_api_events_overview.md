# Events

`fn handle()` is the application entrypoint for handling events, and is passed an [`Event`](/target/doc/zaplib/enum.Event.html). For a detailed view, you can read all of the variants of the `Event` enum. We'll outline different event scenarios below.

## Structuring the `handle` function
Usually, both the top level app and individual components will all have a `handle` function that takes in an `Event`. These functions should:
 * use a `match` expression to handle relevant events for the component
 * pass the `Event` to all child components' `handle` functions
 * call [`cx.request_draw()`](/target/doc/zaplib/struct.Cx.html#method.request_draw) if a redraw is necessitated.
 * call [`cx.request_frame()`](/target/doc/zaplib/struct.Cx.html#method.request_frame) if it should trigger another call to the top level `handle`.

## User input

Mouse and touch input are called "pointers" in Zaplib, represented using [`PointerUp`](/target/doc/zaplib/enum.Event.html#variant.PointerUp), [`PointerDown`](/target/doc/zaplib/enum.Event.html#variant.PointerDown), [`PointerMove`](/target/doc/zaplib/enum.Event.html#variant.PointerMove), and [`PointerScroll`](/target/doc/zaplib/enum.Event.html#variant.PointerScroll), and [`PointerHover`](/target/doc/zaplib/enum.Event.html#variant.PointerHover).

To see if a pointer event is meant for a component, use [`hits_pointer`](/target/doc/zaplib/enum.Event.html#method.hits_pointer). This matches using:
  * a [`ComponentId`](/target/doc/zaplib/struct.ComponentId.html), which is a unique identifier you can assign to your component struct with `ComponentId::default()`.
  * an `Option<Rect>`, which can represent coordinate bounds to match within. This can be manually constructed, but commonly is retrieved from a rendered instance's `Area`, as so:
  ```rust,noplayground
    // Saved somewhere in `draw`
    self.area = cx.add_instances(shader, instance_data);

    // In `handle`
    match event.hits_pointer(cx, self.area.get_rect_for_first_instance(cx)) { ... }
  ```

For processing text input, use [`TextInput`](/target/doc/zaplib/enum.Event.html#variant.TextInput). We also have [`KeyDown`](/target/doc/zaplib/enum.Event.html#variant.KeyDown) and [`KeyUp`](/target/doc/zaplib/enum.Event.html#variant.KeyUp), useful for keyboard based navigation or shortcuts - but do not rely on these for capturing text input. Use [`TextCopy`](/target/doc/zaplib/enum.Event.html#variant.TextCopy) for handling clipboard requests.

You may have different components of your app which take keyboard input. To manage keyboard focus between them, use [`set_key_focus`](/target/doc/zaplib/struct.Cx.html#method.set_key_focus). Like earlier, this matches using [`ComponentId`](/target/doc/zaplib/struct.ComponentId.html).

Then, to see if a keyboard event is meant for a component, use [`hits_keyboard`](/target/doc/zaplib/enum.Event.html#method.hits_keyboard), which will check key focus and skip irrelevant events. It also returns [`KeyFocus`](/target/doc/zaplib/enum.Event.html#variant.KeyFocus) and [`KeyFocusLost`](/target/doc/zaplib/enum.Event.html#variant.KeyFocusLost) if your component should handle focus changes.
