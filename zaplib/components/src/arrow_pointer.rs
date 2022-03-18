use zaplib::*;

pub enum ArrowPointerDirection {
    Up,
    Down,
}

impl ArrowPointerDirection {
    fn shader_float(&self) -> f32 {
        match self {
            ArrowPointerDirection::Up => 0.,
            ArrowPointerDirection::Down => 1.,
        }
    }

    fn apply(&self, pos: Vec2, offset: Vec2) -> Vec2 {
        match self {
            ArrowPointerDirection::Up => pos,
            ArrowPointerDirection::Down => pos - vec2(0., offset.y),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ArrowPointerIns {
    pub base: QuadIns,
    pub color: Vec4,
    pub direction: f32,
}

static MAIN_SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance in_color: vec4;
            instance in_direction: float;

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * vec2(10., 10.));
                if in_direction == 0. {
                    df.triangle(vec2(5., 0.), vec2(10., 10.), vec2(0., 10.));
                } else {
                    df.triangle(vec2(5., 10.), vec2(10., 0.), vec2(0., 0.));
                }
                df.fill(in_color);
                return df.result;
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

impl ArrowPointerIns {
    pub fn draw(cx: &mut Cx, pos: Vec2, color: Vec4, direction: ArrowPointerDirection, size: Vec2) {
        let pos = direction.apply(pos - vec2(0.5 * size.x, 0.), size);
        let rect = Rect { pos, size };
        cx.add_instances(
            &MAIN_SHADER,
            &[ArrowPointerIns { base: QuadIns::from_rect(rect), color, direction: direction.shader_float() }],
        );
    }
}
