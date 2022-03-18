use zaplib::*;
use zaplib_components::*;

#[derive(Clone, Copy)]
#[repr(C)]
struct ShaderQuadIns {
    base: QuadIns,
    counter: f32,
    primary_color: Vec4,
    secondary_color: Vec4,
    mouse: Vec2,
}
impl Default for ShaderQuadIns {
    fn default() -> Self {
        Self {
            base: Default::default(),
            counter: Default::default(),
            primary_color: vec4(151. / 255., 122. / 255., 182. / 255., 1.),
            secondary_color: vec4(213. / 255., 222. / 255., 164. / 255., 1.),
            mouse: Vec2::default(),
        }
    }
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance counter: float;
            instance primary_color: vec4;
            instance secondary_color: vec4;
            instance mouse: vec2;


fn mod289_3(x: vec3) -> vec3 {
return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_4(x: vec4) -> vec4 {
return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x: vec4) -> vec4 {
return mod289_4(((x*34.0)+1.0)*x);
}

fn taylorInvSqrt(r: vec4) -> vec4 {
return 1.79284291400159 - 0.85373472095314 * r;
}

fn snoise3(v: vec3) -> float {
let C = vec2(1.0/6.0, 1.0/3.0) ;
let D = vec4(0.0, 0.5, 1.0, 2.0);

// First corner
let i  = floor(v + dot(v, C.yyy) );
let x0 =   v - i + dot(i, C.xxx) ;

// Other corners
let g = step(x0.yzx, x0.xyz);
let l = 1.0 - g;
let i1 = min( g.xyz, l.zxy );
let i2 = max( g.xyz, l.zxy );

//   x1 = x0 - i1  + 1.0 * C.xxx;
//   x2 = x0 - i2  + 2.0 * C.xxx;
//   x3 = x0 - 1.0 + 3.0 * C.xxx;
let x1 = x0 - i1 + C.xxx;
let x2 = x0 - i2 + C.yyy; // 2.0*C.x = 1/3 = C.y
let x3 = x0 - D.yyy;      // -1.0+3.0*C.x = -0.5 = -D.y

// Permutations
i = mod289_3(i);
let p = permute( permute( permute(
    i.z + vec4(0.0, i1.z, i2.z, 1.0 ))
    + i.y + vec4(0.0, i1.y, i2.y, 1.0 ))
    + i.x + vec4(0.0, i1.x, i2.x, 1.0 ));

// Gradients: 7x7 points over a square, mapped onto an octahedron.
// The ring size 17*17 = 289 is close to a multiple of 49 (49*6 = 294)
let n_ = 0.142857142857; // 1.0/7.0
let ns = n_ * D.wyz - D.xzx;

let j = p - 49.0 * floor(p * ns.z * ns.z);  //  mod(p,7*7)

let x_ = floor(j * ns.z);
let y_ = floor(j - 7.0 * x_ );    // mod(j,N)

let x = x_ *ns.x + ns.yyyy;
let y = y_ *ns.x + ns.yyyy;
let h = 1.0 - abs(x) - abs(y);

let b0 = vec4( x.xy, y.xy );
let b1 = vec4( x.zw, y.zw );

//let s0 = vec4(lessThan(b0,0.0))*2.0 - 1.0;
//let s1 = vec4(lessThan(b1,0.0))*2.0 - 1.0;
let s0 = floor(b0)*2.0 + 1.0;
let s1 = floor(b1)*2.0 + 1.0;
let sh = -step(h, vec4(0.0));

let a0 = b0.xzyw + s0.xzyw*sh.xxyy ;
let a1 = b1.xzyw + s1.xzyw*sh.zzww ;

let p0 = vec3(a0.xy,h.x);
let p1 = vec3(a0.zw,h.y);
let p2 = vec3(a1.xy,h.z);
let p3 = vec3(a1.zw,h.w);

//Normalise gradients
let norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));
p0 *= norm.x;
p1 *= norm.y;
p2 *= norm.z;
p3 *= norm.w;

// Mix final noise value
let m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
m = m * m;
return 42.0 * dot( m*m, vec4( dot(p0,x0), dot(p1,x1),
                        dot(p2,x2), dot(p3,x3) ) );
}

fn fbm(x: vec2) -> float {
let f = 3.0;
let a = 1.0;
let t = 0.0;
let tt = counter;
for i from 0 to 2 {
    t += a * (snoise3(f * vec3(x.x, x.y, tt))/2.0);
    f *= 2.0;
    a *= 0.61259;
}
return t;
}

fn pixel() -> vec4 {
let t = counter;
let fbm_p = fbm(pos);
let q = vec2(fbm_p + mouse);
return fbm(pos + q) > 0. ? primary_color : secondary_color;
}"#
        ),
    ],
    ..Shader::DEFAULT
};
struct ColorSliders {
    label: String,
    slider_r: FloatSlider,
    slider_g: FloatSlider,
    slider_b: FloatSlider,
}

impl ColorSliders {
    fn handle_event(&mut self, cx: &mut Cx, event: &mut Event, color: Vec4) -> Option<Vec4> {
        if let FloatSliderEvent::Change { scaled_value } = self.slider_r.handle(cx, event) {
            Some(Vec4 { x: scaled_value, ..color })
        } else if let FloatSliderEvent::Change { scaled_value } = self.slider_g.handle(cx, event) {
            Some(Vec4 { y: scaled_value, ..color })
        } else if let FloatSliderEvent::Change { scaled_value } = self.slider_b.handle(cx, event) {
            Some(Vec4 { z: scaled_value, ..color })
        } else {
            None
        }
    }

    fn draw_sliders(&mut self, cx: &mut Cx, color: Vec4) {
        cx.begin_padding_box(Padding::top(30.));
        cx.begin_column(Width::Fix(100.), Height::Fill);

        TextIns::draw_walk(cx, &self.label, &TextInsProps::DEFAULT);
        cx.move_draw_pos(0., 20.);
        TextIns::draw_walk(cx, "R", &TextInsProps::DEFAULT);
        self.slider_r.draw(cx, color.x, 0.0, 1.0, None, 1.0, None);
        TextIns::draw_walk(cx, "G", &TextInsProps::DEFAULT);
        self.slider_g.draw(cx, color.y, 0.0, 1.0, None, 1.0, None);
        TextIns::draw_walk(cx, "B", &TextInsProps::DEFAULT);
        self.slider_b.draw(cx, color.z, 0.0, 1.0, None, 1.0, None);

        cx.end_column();
        cx.end_padding_box();
    }
}

struct ShaderEditor {
    component_id: ComponentId,
    window: Window,
    pass: Pass,
    main_view: View,
    quad: ShaderQuadIns,
    quad_area: Area,
    primary_color_picker: ColorSliders,
    secondary_color_picker: ColorSliders,
}

impl ShaderEditor {
    fn new(_cx: &mut Cx) -> Self {
        Self {
            component_id: Default::default(),
            window: Window::default(),
            pass: Pass::default(),
            quad: ShaderQuadIns::default(),
            quad_area: Area::Empty,
            main_view: View::default(),
            primary_color_picker: ColorSliders {
                label: String::from("Primary Color"),
                slider_r: FloatSlider::default(),
                slider_g: FloatSlider::default(),
                slider_b: FloatSlider::default(),
            },
            secondary_color_picker: ColorSliders {
                label: String::from("Secondary Color"),
                slider_r: FloatSlider::default(),
                slider_g: FloatSlider::default(),
                slider_b: FloatSlider::default(),
            },
        }
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let Some(primary_color) = self.primary_color_picker.handle_event(cx, event, self.quad.primary_color) {
            self.quad.primary_color = primary_color;
            cx.request_draw();
        }

        if let Some(secondary_color) = self.secondary_color_picker.handle_event(cx, event, self.quad.secondary_color) {
            self.quad.secondary_color = secondary_color;
            cx.request_draw();
        }

        match event.hits_pointer(cx, self.component_id, self.quad_area.get_rect_for_first_instance(cx)) {
            Event::PointerMove(pm) => {
                self.quad.counter += 0.001;
                self.quad.mouse = pm.rel / self.quad.base.rect_size;
                cx.request_draw();
            }
            _ => (),
        }
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("300"));

        self.main_view.begin_view(cx, LayoutSize::FILL);
        cx.begin_row(Width::Fill, Height::Fill);

        self.primary_color_picker.draw_sliders(cx, self.quad.primary_color);
        self.secondary_color_picker.draw_sliders(cx, self.quad.secondary_color);

        let quad_size: f32 = cx.get_width_left().min(cx.get_height_left());
        cx.move_draw_pos(cx.get_width_left() - quad_size, 0.);
        self.quad.base = QuadIns::from_rect(Rect { pos: cx.get_draw_pos(), size: vec2(quad_size, quad_size) });

        self.quad_area = cx.add_instances(&SHADER, &[self.quad.clone()]);

        cx.end_row();
        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(ShaderEditor);
