use zaplib::*;

static BACKGROUND_SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance min_norm: float;
            instance max_norm: float;
            instance color: vec4;
            instance height_pixels: float;

            // TODO(JP): make it easier to include outside constants in shaders, instead of
            // having to import them through uniforms.
            uniform hor_pad: float;

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);
                let x1 = hor_pad + (rect_size.x-hor_pad*2.) * min_norm;
                let x2 = hor_pad + (rect_size.x-hor_pad*2.) * max_norm;
                df.rect(vec2(x1, rect_size.y/2. - height_pixels/2.), vec2(x2 - x1, height_pixels));
                return df.fill(color);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};
static KNOB_SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance norm_value: float;
            instance hover: float;
            instance down: float;

            // TODO(JP): make it easier to include outside constants in shaders, instead of
            // having to import them through uniforms.
            uniform hor_pad: float;

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);

                let dim = vec2(7., 15.);
                df.box(
                    vec2(
                        hor_pad + (rect_size.x - hor_pad * 2.) * norm_value,
                        rect_size.y / 2.
                    ) - dim / 2.,
                    dim,
                    1.
                );

                let color = mix(mix(#7, #B, hover), #F, down);
                df.fill(color);

                return df.result;
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct FloatSliderIns {
    base: QuadIns,
    norm_value: f32,
    hover: f32,
    down: f32,
}

pub enum FloatSliderEvent {
    Change { scaled_value: f32 },
    DoneChanging,
    None,
}

pub struct FloatSlider {
    component_id: ComponentId,
    scaled_value: f32,
    norm_value: f32,
    animator: Animator,
    min: f32,
    max: f32,
    step: Option<f32>,
    area: Area,
    dragging: bool,
}

pub struct FloatSliderBackgroundRange {
    pub min_scaled: f32,
    pub max_scaled: f32,
    pub color: Vec4,
    pub height_pixels: f32,
}

const DEFAULT_BACKGROUND_RANGES: &[FloatSliderBackgroundRange] = &[FloatSliderBackgroundRange {
    min_scaled: f32::NEG_INFINITY,
    max_scaled: f32::INFINITY,
    color: COLOR_GREY700,
    height_pixels: 8.,
}];

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct FloatSliderBackgroundRangeIns {
    base: QuadIns,
    min_norm: f32,
    max_norm: f32,
    color: Vec4,
    height_pixels: f32,
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // FloatSliderIns::hover
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // FloatSliderIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_HOVER: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // FloatSliderIns::hover
        Track::Float { key_frames: &[(0.0, 1.0)], ease: Ease::DEFAULT },
        // FloatSliderIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // FloatSliderIns::hover
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // FloatSliderIns::down
        Track::Float { key_frames: &[(0.0, 0.0), (1.0, 1.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

/// The horizontal padding on either side of the slider line. We apply this within the shader
/// so that the knob can extend a little bit outside of the line and into the padded area.
const HOR_PAD: f32 = 10.;

impl Default for FloatSlider {
    fn default() -> Self {
        Self {
            component_id: Default::default(),
            norm_value: Default::default(),
            scaled_value: Default::default(),
            animator: Default::default(),
            min: Default::default(),
            max: 1.0,
            step: Default::default(),
            area: Default::default(),
            dragging: Default::default(),
        }
    }
}

impl FloatSlider {
    fn animate(&mut self, cx: &mut Cx) {
        let slider = self.area.get_first_mut::<FloatSliderIns>(cx);
        slider.hover = self.animator.get_float(0);
        slider.down = self.animator.get_float(1);
    }

    pub fn handle_pointer(&mut self, cx: &mut Cx, rel: Vec2) -> FloatSliderEvent {
        let slider = self.area.get_first_mut::<FloatSliderIns>(cx);
        let rect = slider.base.rect();

        let mut norm_value = ((rel.x - HOR_PAD) / (rect.size.x - HOR_PAD * 2.)).max(0.0).min(1.0);
        let mut scaled_value = norm_value * (self.max - self.min) + self.min;
        if self.step.unwrap_or(0.0) > 0.0 {
            scaled_value = (scaled_value / self.step.unwrap_or(1.0)).round() * self.step.unwrap_or(1.0);
            // also adjust norm_value so it is consistent with scaled_value
            norm_value = (scaled_value - self.min) / (self.max - self.min);
        }
        #[allow(clippy::float_cmp)]
        if scaled_value != self.scaled_value {
            self.scaled_value = scaled_value;
            self.norm_value = norm_value;
            slider.norm_value = norm_value;
            return FloatSliderEvent::Change { scaled_value };
        }
        FloatSliderEvent::None
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> FloatSliderEvent {
        if self.animator.handle(cx, event) {
            self.animate(cx);
        }

        match event.hits_pointer(cx, self.component_id, self.area.get_rect_for_first_instance(cx)) {
            Event::PointerHover(pe) => {
                cx.set_hover_mouse_cursor(MouseCursor::Arrow);
                match pe.hover_state {
                    HoverState::In => {
                        self.animator.play_anim(cx, ANIM_HOVER);
                    }
                    HoverState::Out => {
                        self.animator.play_anim(cx, ANIM_DEFAULT);
                    }
                    _ => (),
                }
            }
            Event::PointerDown(pe) => {
                self.animator.play_anim(cx, ANIM_DOWN);
                cx.set_down_mouse_cursor(MouseCursor::Arrow);
                self.dragging = true;
                return self.handle_pointer(cx, pe.rel);
                // lets check where we clicked!
            }
            Event::PointerUp(pe) => {
                if pe.is_over {
                    if pe.input_type.has_hovers() {
                        self.animator.play_anim(cx, ANIM_HOVER);
                    } else {
                        self.animator.play_anim(cx, ANIM_DEFAULT);
                    }
                } else {
                    self.animator.play_anim(cx, ANIM_DEFAULT);
                }
                self.dragging = false;
                return FloatSliderEvent::DoneChanging;
            }
            Event::PointerMove(pe) => return self.handle_pointer(cx, pe.rel),
            Event::PointerScroll(ps) => {
                self.norm_value += ps.scroll.x / 1000.0;
                self.norm_value = self.norm_value.min(1.0).max(0.0);
                self.scaled_value = self.norm_value * (self.max - self.min) + self.min;
                let slider = self.area.get_first_mut::<FloatSliderIns>(cx);
                slider.norm_value = self.norm_value;
                return FloatSliderEvent::Change { scaled_value: self.scaled_value };
            }
            _ => (),
        }
        FloatSliderEvent::None
    }

    pub fn draw(
        &mut self,
        cx: &mut Cx,
        scaled_value: f32,
        min: f32,
        max: f32,
        step: Option<f32>,
        height_scale: f32,
        custom_background_ranges: Option<&[FloatSliderBackgroundRange]>,
    ) {
        if !self.dragging {
            self.scaled_value = scaled_value;
            self.min = min;
            self.max = max;
            self.step = step;
            self.norm_value = (scaled_value - min) / (max - min);
        }

        let rect = cx.add_box(LayoutSize { width: Width::Fill, height: Height::Fix(35.0 * height_scale) });

        let background_ranges = match custom_background_ranges {
            Some(ranges) => ranges,
            None => DEFAULT_BACKGROUND_RANGES,
        };

        let background_data: Vec<FloatSliderBackgroundRangeIns> = background_ranges
            .iter()
            .map(|background_range| {
                let min_scaled = background_range.min_scaled.max(min);
                let max_scaled = background_range.max_scaled.min(max);
                FloatSliderBackgroundRangeIns {
                    base: QuadIns::from_rect(rect),
                    min_norm: (min_scaled - min) / (max - min),
                    max_norm: (max_scaled - min) / (max - min),
                    color: background_range.color,
                    height_pixels: background_range.height_pixels,
                }
            })
            .collect();
        let background_area = cx.add_instances(&BACKGROUND_SHADER, &background_data);
        background_area.write_user_uniforms(cx, HOR_PAD);

        self.area = cx.add_instances(
            &KNOB_SHADER,
            &[FloatSliderIns { base: QuadIns::from_rect(rect), norm_value: self.norm_value, ..Default::default() }],
        );
        self.area.write_user_uniforms(cx, HOR_PAD);

        self.animator.draw(cx, ANIM_DEFAULT);
        self.animate(cx);
    }
}
