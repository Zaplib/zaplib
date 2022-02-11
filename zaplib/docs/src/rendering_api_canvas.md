# Canvas

On the web, we need a `<canvas>` element somewhere to draw on. Currently, this element must have the following properties:
1. Span the entire page (absolutely positioned).
2. Not used for any other rendering.
3. There can only be one such canvas.
4. It may be layered at any z-index: either behind other elements, on top of it, or in the middle (e.g. behind buttons and popovers, but in front of backgrounds).

You can specify the canvas in a few ways:
1. Use `zaplib.initialize({ defaultStyles: true })`, which automatically adds a canvas to the body of the page.
2. Pass it in using `zaplib.initialize({ canvas })`. In this case some styles will automatically be applied to the canvas (through the `zaplib_canvas` CSS class), but you can override these yourself.

Interoperation with existing DOM elements is still limited, but it is possible to add `id="zaplib_js_root"` to the root element that contains your other DOM elements in order to prevent Zaplib from handling events that are already captured by your application.
