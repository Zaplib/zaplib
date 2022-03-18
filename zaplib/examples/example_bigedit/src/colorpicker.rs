use zaplib::*;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct ColorPickerIns {
    base: QuadIns,
    hue: f32,
    sat: f32,
    val: f32,
    hover: f32,
    down: f32,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance hue: float;
            instance sat: float;
            instance val: float;
            instance hover: float;
            instance down: float;

            fn circ_to_rect(u: float, v: float) -> vec2 {
                let u2 = u * u;
                let v2 = v * v;
                return vec2(
                    0.5 * sqrt(2. + 2. * sqrt(2.) * u + u2 - v2) -
                    0.5 * sqrt(2. - 2. * sqrt(2.) * u + u2 - v2),
                    0.5 * sqrt(2. + 2. * sqrt(2.) * v - u2 + v2) -
                    0.5 * sqrt(2. - 2. * sqrt(2.) * v - u2 + v2)
                );
            }

            fn pixel() -> vec4 {
                let rgbv = hsv2rgb(vec4(hue, sat, val, 1.));
                let w = rect_size.x;
                let h = rect_size.y;
                let df = Df::viewport(pos * vec2(w, h));
                let c = vec2(w, h) * 0.5;

                let radius = w * 0.37;
                let inner = w * 0.28;

                df.hexagon(c, w * 0.45);
                df.hexagon(c, w * 0.4);
                df.subtract();
                let ang = atan(pos.x * w - c.x, 0.0001 + pos.y * h - c.y) / PI * 0.5 - 0.33333;
                df.fill(hsv2rgb(vec4(ang, 1.0, 1.0, 1.0)));
                df.new_path();

                let rsize = inner / sqrt(2.0);
                df.rect(c - rsize, vec2(rsize * 2.0));

                let norm_rect = (vec2(pos.x * w, pos.y * h) - (c - inner)) / (2. * inner);
                let circ = clamp(circ_to_rect(norm_rect.x * 2. - 1., norm_rect.y * 2. - 1.), vec2(-1.), vec2(1.));

                df.fill(hsv2rgb(vec4(hue, (circ.x * .5 + .5), 1. - (circ.y * .5 + .5), 1.)));
                df.new_path();

                let col_angle = (hue + .333333) * 2. * PI;
                let circle_puk = vec2(sin(col_angle) * radius, cos(col_angle) * radius) + c;

                let rect_puk = c + vec2(sat * 2. * rsize - rsize, (1. - val) * 2. * rsize - rsize);

                let color = mix(mix(#3, #E, hover), #F, down);
                let puck_size = 0.1 * w;
                df.circle(rect_puk, puck_size);
                df.rect(c - rsize, vec2(rsize * 2.0));
                df.intersect();
                df.fill(color);
                df.new_path();
                df.circle(rect_puk, puck_size - 1. - 2. * hover + down);
                df.rect(c - rsize, vec2(rsize * 2.0));
                df.intersect();
                df.fill(rgbv);
                df.new_path();

                df.circle(circle_puk, puck_size);
                df.fill(color);
                df.new_path();
                df.circle(circle_puk, puck_size - 1. - 2. * hover + down);
                df.fill(rgbv);
                df.new_path();

                return df.result;
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

pub enum ColorPickerEvent {
    Change { hsva: Vec4 },
    DoneChanging,
    None,
}

#[derive(Default)]
pub struct ColorPicker {
    component_id: ComponentId,
    size: f32,
    area: Area,
    animator: Animator,
    drag_mode: ColorPickerDragMode,
}

#[derive(Clone, Debug, PartialEq)]
enum ColorPickerDragMode {
    Wheel,
    Rect,
    None,
}

impl Default for ColorPickerDragMode {
    fn default() -> Self {
        ColorPickerDragMode::None
    }
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // ColorPickerIns::hover
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // ColorPickerIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_HOVER: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // ColorPickerIns::hover
        Track::Float { key_frames: &[(0.0, 1.0)], ease: Ease::DEFAULT },
        // ColorPickerIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // ColorPickerIns::hover
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // ColorPickerIns::down
        Track::Float { key_frames: &[(0.0, 0.0), (1.0, 1.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

impl ColorPicker {
    fn animate(&mut self, cx: &mut Cx) {
        let color_picker = self.area.get_first_mut::<ColorPickerIns>(cx);
        color_picker.hover = self.animator.get_float(0);
        color_picker.down = self.animator.get_float(1);
    }

    fn handle_pointer(&mut self, cx: &mut Cx, rel: Vec2) -> ColorPickerEvent {
        let color_picker = self.area.get_first_mut::<ColorPickerIns>(cx);
        let size = color_picker.base.rect_size.x;
        let vx = rel.x - 0.5 * size;
        let vy = rel.y - 0.5 * size;
        let rsize = (size * 0.28) / 2.0f32.sqrt();
        let last_hue = color_picker.hue;
        let last_sat = color_picker.sat;
        let last_val = color_picker.val;
        match self.drag_mode {
            ColorPickerDragMode::Rect => {
                color_picker.sat = ((vx + rsize) / (2.0 * rsize)).clamp(0.0, 1.0);
                color_picker.val = 1.0 - ((vy + rsize) / (2.0 * rsize)).clamp(0.0, 1.0);
            }
            ColorPickerDragMode::Wheel => {
                color_picker.hue = vx.atan2(vy) / std::f32::consts::PI * 0.5 - 0.33333;
            }
            _ => (),
        }

        if last_hue != color_picker.hue || last_sat != color_picker.sat || last_val != color_picker.val {
            return ColorPickerEvent::Change {
                hsva: Vec4 { x: color_picker.hue, y: color_picker.sat, z: color_picker.val, w: 1.0 },
            };
        }
        ColorPickerEvent::None
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ColorPickerEvent {
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
                let color_picker = self.area.get_first::<ColorPickerIns>(cx);
                let size = color_picker.base.rect_size.x;
                let rsize = (size * 0.28) / 2.0f32.sqrt();
                let vx = pe.rel.x - 0.5 * size;
                let vy = pe.rel.y - 0.5 * size;
                if vx >= -rsize && vx <= rsize && vy >= -rsize && vy <= rsize {
                    self.drag_mode = ColorPickerDragMode::Rect;
                } else if vx >= -0.5 * size && vx <= 0.5 * size && vy >= -0.5 * size && vy <= 0.5 * size {
                    self.drag_mode = ColorPickerDragMode::Wheel;
                } else {
                    self.drag_mode = ColorPickerDragMode::None;
                }
                return self.handle_pointer(cx, pe.rel);
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
                self.drag_mode = ColorPickerDragMode::None;
                return ColorPickerEvent::DoneChanging;
            }
            Event::PointerMove(pe) => return self.handle_pointer(cx, pe.rel),
            _ => (),
        }
        ColorPickerEvent::None
    }

    pub fn draw(&mut self, cx: &mut Cx, hsva: Vec4, height_scale: f32) {
        // i wanna draw a wheel with 'width' set but height a fixed height.
        self.size = cx.get_box_rect().size.x;

        let rect = cx.add_box(LayoutSize { width: Width::Fill, height: Height::Fix(self.size * height_scale) });

        self.area = cx.add_instances(
            &SHADER,
            &[ColorPickerIns { base: QuadIns::from_rect(rect), hue: hsva.x, sat: hsva.y, val: hsva.z, ..Default::default() }],
        );

        self.animator.draw(cx, ANIM_DEFAULT);
        self.animate(cx);
    }
}
