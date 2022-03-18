use crate::debug_log::DebugLog;
use crate::*;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct BorderIns {
    quad: QuadIns,
}

/// Draws small border around the provided rect with transparent background
static BORDER_SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            fn pixel() -> vec4 {
                let transparent = vec4(0.0, 0.0, 0.0, 0.0);
                let m = 1.0;
                let abs_pos = pos * rect_size;
                if abs_pos.x < m || abs_pos.y < m || abs_pos.x > rect_size.x - m || abs_pos.y > rect_size.y - m {
                    return vec4(1., 1., 0.5, 1.0);
                } else {
                    return transparent;
                }
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default, Clone)]
pub struct Debugger {
    area: Area,
}

impl Debugger {
    pub fn new() -> Self {
        Self { area: Area::Empty }
    }

    fn draw_border(&mut self, cx: &mut Cx, rect: Rect) {
        let data = BorderIns { quad: QuadIns { rect_pos: rect.pos, rect_size: rect.size, draw_depth: 0.0 } };
        self.area = cx.add_instances(&BORDER_SHADER, &[data]);
        let bg = self.area.get_first_mut::<BorderIns>(cx);
        bg.quad.rect_pos = rect.pos;
        bg.quad.rect_size = rect.size;
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        // TODO(Dmitry): unnecessary copying here
        let logs = cx.debug_logs.clone();
        for log in logs {
            match log {
                DebugLog::EndBox { rect } => {
                    self.draw_border(cx, rect);
                }
            }
        }
    }
}
