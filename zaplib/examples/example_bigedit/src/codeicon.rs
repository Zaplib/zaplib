use zaplib::*;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CodeIconIns {
    base: QuadIns,
    icon_type: f32,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance icon_type: float;
            fn pixel() -> vec4 {
                if abs(icon_type - 5.) < 0.1 { //Wait
                    let df = Df::viewport(pos * vec2(10., 10.)); // * vec2(w, h));
                    df.circle(vec2(5.), 4.);
                    df.fill(#ffa500);
                    df.stroke(#be, 0.5);
                    df.new_path();
                    df.move_to(vec2(3., 5.));
                    df.line_to(vec2(3., 5.));
                    df.move_to(vec2(5.));
                    df.line_to(vec2(5.));
                    df.move_to(vec2(7., 5.));
                    df.line_to(vec2(7., 5.));
                    df.stroke(#0, 0.8);
                    return df.result;
                }
                if abs(icon_type - 4.) < 0.1 { //OK
                    let df = Df::viewport(pos * vec2(10., 10.)); // * vec2(w, h));
                    df.circle(vec2(5.), 4.);
                    df.fill(#5);
                    df.stroke(#5, 0.5);
                    df.new_path();
                    let sz = 1.;
                    df.move_to(vec2(5.));
                    df.line_to(vec2(5.));
                    df.stroke(#a, 0.8);
                    return df.result;
                }
                else if abs(icon_type - 3.) < 0.1 { // Error
                    let df = Df::viewport(pos * vec2(10., 10.)); // * vec2(w, h));
                    df.circle(vec2(5.), 4.);
                    df.fill(#c00);
                    df.stroke(#be, 0.5);
                    df.new_path();
                    let sz = 1.;
                    df.move_to(vec2(5. - sz));
                    df.line_to(vec2(5. + sz));
                    df.move_to(vec2(5. - sz, 5. + sz));
                    df.line_to(vec2(5. + sz, 5. - sz));
                    df.stroke(#0, 0.8);
                    return df.result;
                }
                else if abs(icon_type - 2.) < 0.1 { // Warning
                    let df = Df::viewport(pos * vec2(10., 10.)); // * vec2(w, h));
                    df.move_to(vec2(5., 1.));
                    df.line_to(vec2(9.));
                    df.line_to(vec2(1., 9.));
                    df.close_path();
                    df.fill(vec4(253.0 / 255.0, 205.0 / 255.0, 59.0 / 255.0, 1.0));
                    df.stroke(#be, 0.5);
                    df.new_path();
                    df.move_to(vec2(5., 3.5));
                    df.line_to(vec2(5., 5.25));
                    df.stroke(#0, 0.8);
                    df.new_path();
                    df.move_to(vec2(5., 7.25));
                    df.line_to(vec2(5., 7.5));
                    df.stroke(#0, 0.8);
                    return df.result;
                }
                else { // Panic
                    let df = Df::viewport(pos * vec2(10., 10.)); // * vec2(w, h));
                    df.move_to(vec2(5., 1.));
                    df.line_to(vec2(9.));
                    df.line_to(vec2(1., 9.));
                    df.close_path();
                    df.fill(#c00);
                    df.stroke(#be, 0.5);
                    df.new_path();
                    let sz = 1.;
                    df.move_to(vec2(5. - sz, 6.25 - sz));
                    df.line_to(vec2(5. + sz, 6.25 + sz));
                    df.move_to(vec2(5. - sz, 6.25 + sz));
                    df.line_to(vec2(5. + sz, 6.25 - sz));
                    df.stroke(#f, 0.8);

                    return df.result;
                }
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

pub enum CodeIconType {
    Panic,
    Warning,
    Error,
    Ok,
    Wait,
}

impl CodeIconType {
    fn shader_float(&self) -> f32 {
        match self {
            CodeIconType::Panic => 1.,
            CodeIconType::Warning => 2.,
            CodeIconType::Error => 3.,
            CodeIconType::Ok => 4.,
            CodeIconType::Wait => 5.,
        }
    }
}

impl CodeIconIns {
    pub fn draw(cx: &mut Cx, icon_type: CodeIconType) {
        cx.begin_padding_box(Padding { l: 0., t: 0.5, r: 4., b: 0. });

        let rect = cx.add_box(LayoutSize { width: Width::Fix(14.0), height: Height::Fix(14.0) });

        cx.add_instances(&SHADER, &[CodeIconIns { base: QuadIns::from_rect(rect), icon_type: icon_type.shader_float() }]);
        cx.end_padding_box();
    }
}
