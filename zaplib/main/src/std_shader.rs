//! Collection of standard [`Shader`] functions.

use crate::*;

impl Cx {
    /// Collection of standard [`Shader`] functions.
    // Based on https://www.shadertoy.com/view/lslXW8
    pub const STD_SHADER: CodeFragment = code_fragment!(
        r#"
        // See [`PassUniforms`] for documentation on these fields.
        uniform camera_projection: mat4 in pass;
        uniform camera_view: mat4 in pass;
        uniform inv_camera_rot: mat4 in pass;
        uniform dpi_factor: float in pass;
        uniform dpi_dilate: float in pass;

        // See [`DrawUniforms`] for documentation on these fields.
        uniform draw_clip: vec4 in draw;
        uniform draw_scroll: vec2 in draw;
        uniform draw_local_scroll: vec2 in draw;
        uniform draw_zbias: float in draw;

        const PI: float = 3.141592653589793;
        const E: float = 2.718281828459045;
        const LN2: float = 0.6931471805599453;
        const LN10: float = 2.302585092994046;
        const LOG2E: float = 1.4426950408889634;
        const LOG10E: float = 0.4342944819032518;
        const SQRT1_2: float = 0.70710678118654757;
        const TORAD: float = 0.017453292519943295;
        const GOLDEN: float = 1.618033988749895;

        // The current distance field
        struct Df {
            pos: vec2,
            result: vec4,
            last_pos: vec2,
            start_pos: vec2,
            shape: float,
            clip: float,
            has_clip: float,
            old_shape: float,
            blur: float,
            aa: float,
            scale: float,
            field: float
        }

        impl Math{
            // Rotate vector `v` by radians `a`
            fn rotate_2d(v: vec2, a: float)->vec2 {
                let ca = cos(a);
                let sa = sin(a);
                return vec2(v.x * ca - v.y * sa, v.x * sa + v.y * ca);
            }
        }

        //http://gamedev.stackexchange.com/questions/59797/glsl-shader-change-hue-saturation-brightness
        fn hsv2rgb(c: vec4) -> vec4 {
            let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
            let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
            return vec4(c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y), c.w);
        }

        fn rgb2hsv(c: vec4) -> vec4 {
            let K: vec4 = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
            let p: vec4 = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
            let q: vec4 = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

            let d: float = q.x - min(q.w, q.y);
            let e: float = 1.0e-10;
            return vec4(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x, c.w);
        }

        impl Df {
            // Creates a distance field with the current position
            fn viewport(pos: vec2) -> Df {
                let df: Df;
                df.pos = pos;
                df.result = vec4(0.);
                df.last_pos = vec2(0.);
                df.start_pos = vec2(0.);
                df.shape = 1e+20;
                df.clip = -1e+20;
                df.has_clip = 0.0;
                df.old_shape = 1e+20;
                df.blur = 0.00001;
                df.aa = Df::antialias(pos);
                df.scale = 1.0;
                df.field = 0.0;
                return df;
            }

            // Creates a distance field with the current position, matching pixel scale
            fn viewport_px(pos: vec2) -> Df {
                return Df::viewport(pos * dpi_factor);
            }

            // Adds a new field value to the current distance field
            fn add_field(inout self, field: float) {
                self.field = field / self.scale;
                self.old_shape = self.shape;
                self.shape = min(self.field, self.shape);
            }

            // Adds a clip mask to the current distance field
            fn add_clip(inout self, d: float) {
                d = d / self.scale;
                self.clip = max(self.clip, d);
                self.has_clip = 1.;
            }

            fn antialias(p: vec2) -> float {
                return 1.0 / length(vec2(length(dFdx(p)), length(dFdy(p))));
            }

            // Translate a specified offset
            fn translate(inout self, offset: vec2) -> vec2 {
                self.pos -= offset;
                return self.pos;
            }

            // Rotate by `a` radians around pivot
            fn rotate(inout self, a: float, pivot: vec2) {
                self.pos = Math::rotate_2d(self.pos - pivot, -a) + pivot;
            }

            // Uniformly scale by factor `f` around `pivot`
            fn scale(inout self, f: float, pivot: vec2) {
                self.scale *= f;
                self.pos = (self.pos - pivot) * f + pivot;
            }

            // Sets clear color. Useful for specifying background colors before
            // rendering a path.
            fn clear(inout self, color: vec4) {
                self.write_color(color, 1.0);
            }

            // Calculate antialiasing blur
            // Private function
            fn calc_blur(inout self, w: float) -> float {
                let wa = clamp(-w * self.aa, 0.0, 1.0);
                let wb = 1.0;
                if self.blur > 0.001 {
                    wb = clamp(-w / self.blur, 0.0, 1.0);
                }
                return wa * wb;
            }

            // Clears path in current distance field.
            fn new_path(inout self) -> vec4 {
                self.old_shape = self.shape = 1e+20;
                self.clip = -1e+20;
                self.has_clip = 0.;
                return self.result;
            }

            // Writes a color to the distance field, using premultiplied alpha
            // Private function. Users should instead use `clear`, `fill`, `stroke`.
            fn write_color(inout self, src: vec4, w: float) -> vec4{
                let src_a = src.a * w;
                self.result = src * src_a + (1. - src_a) * self.result;
                return self.result;
            }

            // Fills the current path with `color`.
            fn fill(inout self, color: vec4) -> vec4 {
                let f = self.calc_blur(self.shape);
                self.write_color(color, f);
                if self.has_clip > 0. {
                    self.write_color(color, self.calc_blur(self.clip));
                }
                return self.result;
            }

            // Strokes the current path with `color` with a pixel width of `width`.
            fn stroke(inout self, color: vec4, width: float) -> vec4 {
                let f = self.calc_blur(abs(self.shape) - width / self.scale);
                return self.write_color(color, f);
            }

            // Updates the current path by summing colors in `width`
            // with the provided one.
            fn glow(inout self, color: vec4, width: float) -> vec4 {
                let f = self.calc_blur(abs(self.shape) - width / self.scale);
                let source = vec4(color.rgb * color.a, color.a);
                let dest = self.result;
                self.result = vec4(source.rgb * f, 0.) + dest;
                return self.result;
            }

            // Set field to the union of the current and previous field.
            fn union(inout self) {
                self.old_shape = self.shape = min(self.field, self.old_shape);
            }

            // Set field to the intersection of the current and previous field.
            fn intersect(inout self) {
                self.old_shape = self.shape = max(self.field, self.old_shape);
            }

            // Subtract current field from previous.
            fn subtract(inout self) {
                self.old_shape = self.shape = max(-self.field, self.old_shape);
            }

            // Interpolate current field and previous with factor k
            fn blend(inout self, k: float) {
                self.old_shape = self.shape = mix(self.old_shape, self.field, k);
            }

            // Renders a circle at p with radius r
            fn circle(inout self, p: vec2, r: float) {
                let c = self.pos - p;
                self.add_field(length(c) - r);
            }

            // Render an arc at p with radius r between angles angle_start and angle_end.
            fn arc(inout self, p: vec2, r: float, angle_start: float, angle_end: float) {
                let c = self.pos - p;
                let angle = mod(atan(c.x, -c.y) + 2.*PI, 2.*PI);
                let d = max( angle_start - angle, angle - angle_end );
                let len = max(length(c) * d, length(c) - r);
                self.add_field(len / self.scale);
            }

            // Render a box with rounded corners at p with dimensions d.
            // Use `r` to indicate the corner radius - if r is less than 1, render a basic
            // rectangle. If r is bigger than min(w, h), the result will be a circle.
            fn box(inout self, pos: vec2, size: vec2, r: float) {
                let half_size = 0.5 * size;
                let center = pos + half_size;
                r = min(r, min(size.x, size.y));
                half_size -= r;
                let dist_from_edge = abs(center - self.pos) - half_size;
                let dneg = min(dist_from_edge, 0.);
                let dpos = max(dist_from_edge, 0.);
                let df = max(dneg.x, dneg.y) + length(dpos);
                self.add_field(df - r);
            }

            // Render a rectangle at p with dimensions d.
            fn rect(inout self, p: vec2, d: vec2) {
                self.box(p, d, 0.);
            }

            // Render a triangle between points p0, p1, p2.
            fn triangle(inout self, p0: vec2, p1: vec2, p2: vec2) {
                let e0 = p1 - p0;
                let e1 = p2 - p1;
                let e2 = p0-p2;

                let v0 = self.pos - p0;
                let v1 = self.pos - p1;
                let v2 = self.pos - p2;

                let pq0 = v0 - e0 * clamp(dot(v0, e0) / dot(e0, e0), 0.0, 1.0);
                let pq1 = v1 - e1 * clamp(dot(v1, e1) / dot(e1, e1), 0.0, 1.0);
                let pq2 = v2 - e2 * clamp(dot(v2, e2) / dot(e2, e2), 0.0, 1.0);

                let s = sign(e0.x * e2.y - e0.y * e2.x);
                let d = min(min(vec2(dot(pq0, pq0), s*(v0.x * e0.y - v0.y * e0.x)),
                        vec2(dot(pq1, pq1), s * (v1.x * e1.y - v1.y * e1.x))),
                        vec2(dot(pq2, pq2), s * (v2.x * e2.y - v2.y * e2.x)));

                self.add_field(-sqrt(d.x) * sign(d.y));
            }

            // Render a hexagon at p with side length r.
            fn hexagon(inout self, p: vec2, r: float) {
                let dx = abs(p.x - self.pos.x) * 1.15;
                let dy = abs(p.y - self.pos.y);
                self.add_field(max(dy + cos(60.0 * TORAD) * dx - r, dx - r));
            }

            // Move to p in current path, not drawing from current position.
            fn move_to(inout self, p: vec2) {
                self.last_pos =
                self.start_pos = p;
            }

            // Render a line to p from current position.
            fn line_to(inout self, p: vec2) {
                let pa = self.pos - self.last_pos;
                let ba = p - self.last_pos;
                let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
                let s = sign(pa.x * ba.y - pa.y * ba.x);
                self.field = length(pa - ba * h) / self.scale;
                self.old_shape = self.shape;
                self.shape = min(self.shape, self.field);
                self.clip = max(self.clip, self.field * s);
                self.has_clip = 1.0;
                self.last_pos = p;
            }

            // End the current field by rendering a line back to the start point
            fn close_path(inout self) {
                self.line_to(self.start_pos);
            }
        }
    "#
    );
}
