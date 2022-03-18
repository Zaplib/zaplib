//! Convenient way of drawing colored rectangles, built on top of [`QuadIns`].

use zaplib::*;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct BackgroundIns {
    quad: QuadIns,
    color: Vec4,
    radius: f32,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance color: vec4;
            instance radius: float;
            fn pixel() -> vec4 {
                // TODO(JP): Giant hack! We should just be able to call df.box with radius=0
                // and then df.fill, but df.box with radius=0 seems totally broken, and even
                // using df.rect with df.fill seems to leave a gap around the border..
                if radius < 0.001 {
                    return vec4(color.rgb*color.a, color.a);
                }

                let df = Df::viewport(pos * rect_size);
                df.box(vec2(0.), rect_size, radius);
                return df.fill(color);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub struct Background {
    area: Area,
    radius: f32,
    draw_depth: f32,
}

impl Background {
    pub fn set_color(&mut self, cx: &mut Cx, color: Vec4) {
        let bg = self.area.get_first_mut::<BackgroundIns>(cx);
        bg.color = color;
    }
    #[must_use]
    pub fn with_draw_depth(self, draw_depth: f32) -> Self {
        Self { draw_depth, ..self }
    }
    #[must_use]
    pub fn with_radius(self, radius: f32) -> Self {
        Self { radius, ..self }
    }

    /// Calls [`Self::draw`] without having to pass in a [`Rect`] immediately. We will overwrite
    /// the coordinates in the shader directly in [`Background::end_draw`].
    ////
    /// This is useful for if you need to draw a quad in the background, since in that case you have
    /// to draw the quad first before drawing the content (otherwise it would sit on top of the
    /// content!), but you might not know the dimensions yet. In [`Background::end_draw`] we
    /// get the dimensions of the content from [`Cx::end_row`] and set this directly using
    /// [`Area::get_first_mut`].
    pub fn begin_draw(&mut self, cx: &mut Cx, width: Width, height: Height, color: Vec4) {
        self.draw(cx, Rect::default(), color);
        cx.begin_row(width, height);
    }

    /// See [`Background::begin_draw`].
    pub fn end_draw(&mut self, cx: &mut Cx) {
        let rect = cx.end_row();
        let bg = self.area.get_first_mut::<BackgroundIns>(cx);
        bg.quad.rect_pos = rect.pos;
        bg.quad.rect_size = rect.size;
    }

    /// Get the [`Area`].
    pub fn area(&self) -> Area {
        self.area
    }

    /// Manually set the [`Area`].
    pub fn set_area(&mut self, area: Area) {
        self.area = area;
    }

    /// Draw the background.
    pub fn draw(&mut self, cx: &mut Cx, rect: Rect, color: Vec4) {
        let data = BackgroundIns { quad: QuadIns::from_rect(rect).with_draw_depth(self.draw_depth), color, radius: self.radius };
        self.area = cx.add_instances(&SHADER, &[data]);
    }

    /// Draw the background, but make it sticky with respect to scrolling. Not typically recommended.
    pub fn draw_with_scroll_sticky(&mut self, cx: &mut Cx, rect: Rect, color: Vec4) {
        let data = BackgroundIns { quad: QuadIns::from_rect(rect).with_draw_depth(self.draw_depth), color, radius: self.radius };
        self.area = cx.add_instances_with_scroll_sticky(&SHADER, &[data], true, true);
    }
}
