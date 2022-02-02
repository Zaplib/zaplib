use crate::*;
use zaplib::*;

struct PerfPoint {
    timestamp: f64,
    value: f64,
}

const AVERAGE_RANGE: f64 = 5.;
const TOP_PADDING: f32 = 5.;
const NUM_SAMPLES: usize = 300;

#[repr(C)]
struct FpsCounterUniforms {
    sample_length: f32,
    max_fps: f32,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            texture texture: texture2D;
            uniform sample_length: float;
            uniform max_fps: float;

            const line_width: float = 0.05;

            fn plot(position: vec2, point: float) -> float {
                return smoothstep(point - line_width, point, position.y) - smoothstep(point, point + line_width, position.y);
            }

            fn pixel() -> vec4 {
                // Flip Y
                pos.y = 1. - pos.y;

                // Normalize across number of samples
                let texture_position = vec2(pos.x * sample_length/300., 0.);

                let color = vec3(plot(pos, floor(sample2d(texture, texture_position).x*255.)/max_fps));
                return vec4(color, 1.0);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub struct FpsCounter {
    points: Vec<PerfPoint>,
    enable_button: Button,
    enabled: bool,
    texture: Texture,
}

impl FpsCounter {
    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let Event::NextFrame = event {
            if self.enabled {
                let timestamp = cx.last_event_time;
                let last_draw_time = self.points.last().unwrap().timestamp;
                let value = timestamp - last_draw_time;
                self.points.push(PerfPoint { timestamp, value });

                // Remove points not within AVERAGE_RANGE
                let mut range_to_del = 0;
                while self.points.get(range_to_del).unwrap().timestamp + AVERAGE_RANGE < timestamp {
                    range_to_del += 1;
                }
                if range_to_del > 0 {
                    self.points.drain(0..range_to_del);
                }

                cx.request_draw();
                cx.request_next_frame();
            }
        }

        if let ButtonEvent::Clicked = self.enable_button.handle(cx, event) {
            self.enabled = !self.enabled;
            if self.enabled {
                self.points = vec![PerfPoint { timestamp: cx.last_event_time, value: 1. / 60.0 }];
                cx.request_next_frame();
            }
            cx.request_draw();
        }
    }

    pub fn size(&self) -> f32 {
        if self.enabled {
            240.
        } else {
            80.
        }
    }

    fn draw_graph(&mut self, cx: &mut Cx) {
        let texture_handle = self.texture.get_with_dimensions(cx, NUM_SAMPLES, 1);
        let image = texture_handle.get_image_mut(cx);
        let mut max_fps: f64 = -1.;
        for i in 0..NUM_SAMPLES {
            match self.points.get(i) {
                Some(v) => {
                    let fps = 1. / v.value;
                    max_fps = max_fps.max(fps);
                    *(image.get_mut(i).unwrap()) = fps as u32;
                }
                None => {
                    break;
                }
            }
        }

        let area = cx.add_instances(&SHADER, &[QuadIns::from_rect(cx.get_box_rect())]);
        area.write_texture_2d(cx, "texture", texture_handle);
        area.write_user_uniforms(cx, FpsCounterUniforms { sample_length: self.points.len() as f32, max_fps: max_fps as f32 });
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        cx.begin_row(Width::Fill, Height::Fill);
        let third_of_width = cx.get_width_left() / 3.;
        if self.enabled {
            cx.begin_row(Width::Fix(third_of_width), Height::Fill);
            self.draw_graph(cx);
            cx.end_row();

            cx.begin_row(Width::Fix(third_of_width), Height::Fill);
            let value = self.points.last().unwrap().value;
            let fps = if value == 0. { 0. } else { 1. / value };
            let avg: f64 = self.points.len() as f64 / self.points.iter().map(|p| p.value).sum::<f64>();
            TextIns::draw_str(
                cx,
                &format!("{:.1} fps", fps),
                cx.get_box_origin() + Vec2 { x: 0., y: TOP_PADDING },
                &TextInsProps::DEFAULT,
            );
            TextIns::draw_str(
                cx,
                &format!("{:.1} avg", avg),
                cx.get_box_origin() + Vec2 { x: 0., y: TOP_PADDING + cx.get_height_left() / 2. },
                &TextInsProps::DEFAULT,
            );
            cx.end_row();
        }

        cx.begin_row(Width::Fix(third_of_width), Height::Fill);
        cx.begin_center_y_align();
        self.enable_button.draw(cx, if self.enabled { "Hide FPS" } else { "Show FPS" });
        cx.end_center_y_align();
        cx.end_row();

        cx.end_row();
    }
}
