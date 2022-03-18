use zaplib::*;
use zaplib_components::*;

mod mprstokenizer;
use mprstokenizer::*;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct ShaderQuadIns {
    base: QuadIns,
    knobs: [Vec2; 3],
    knobs_mirror: [Vec2; 3],
    down: [f32; 3],
    show_knobs: f32,
    time: f32,
}

// Keep in sync with KNOB_RADIUS in shader.
const KNOB_RADIUS: f32 = 10.;
const ANIMATION_LENGTH: f64 = 1.5;

const INSTANCE_CODE: CodeFragment = code_fragment!(
    r#"
    instance knob1: vec2;
    instance knob2: vec2;
    instance knob3: vec2;
    instance knob1_mirror: vec2;
    instance knob2_mirror: vec2;
    instance knob3_mirror: vec2;
    instance down: vec3;
    instance show_knobs: float;
    instance time: float;

    const KNOB_RADIUS: float = 10.;

    fn pixel() -> vec4 {
        let points: vec2[7];
        points[0] = knob1 * rect_size;
        points[1] = knob2 * rect_size;
        points[2] = knob3 * rect_size;
        points[3] = knob1_mirror * rect_size;
        points[4] = knob2_mirror * rect_size;
        points[5] = knob3_mirror * rect_size;
        points[6] = knob1 * rect_size;

        let df = Df::viewport(pos * rect_size);
        df.result = min(#ffff, shader(df, points));

        for i from 0 to 3 {
            df.new_path();
            df.circle(points[i], KNOB_RADIUS);
            df.stroke(vec4(mix(#f00, #f60, down[i]).rgb, show_knobs), 2.0);
        }

        df.new_path();
        for i from 3 to 6 {
            df.circle(points[i], KNOB_RADIUS);
        }
        df.stroke(vec4(#a.rgb, show_knobs), 2.0);

        return df.result;
    }
    "#
);
// Inspired by https://www.shadertoy.com/view/WdK3Dz
const CODE: CodeFragment = code_fragment!(
    r#"const LINES: int = 3;
const LINE_BASE_LENGTH: float = 0.3;

fn shader(df: Df, points: vec2[7]) -> vec4 {
    let total = 0.0;
    for i from 0 to 6 {
        total += length(points[i+1]-points[i]);
        df.move_to(points[i]);
        df.line_to(points[i+1]);
    }
    let result = pow(10./df.shape, 4.0) * vec4(1.0,0.7,0.3, 0.2);

    df.new_path();
    for l from 0 to LINES {
        let dist_so_far = 0.0;
        for i from 0 to 6 {
            let start = (time + float(l)/float(LINES)) * total;
            let end = (time + float(l)/float(LINES) +
                LINE_BASE_LENGTH / float(LINES)) * total;
            let len = length(points[i+1]-points[i]);
            if end >= total && start > dist_so_far + len {
                start -= total;
                end -= total;
            }

            let from_t = clamp((start-dist_so_far)/len, 0., 1.);
            let to_t = clamp((end-dist_so_far)/len, 0., 1.);
            if (to_t > 0.0 && from_t < 1.0) {
                df.move_to(mix(points[i], points[i+1], from_t));
                df.line_to(mix(points[i], points[i+1], to_t));
            }
            dist_so_far += len;
        }
    }
    return result + pow(10./df.shape, 1.7) * vec4(1.0,0.7,0.3,1.0);
}"#
);

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[Cx::STD_SHADER, QuadIns::SHADER, INSTANCE_CODE, CODE],

    ..Shader::DEFAULT
};

struct LogoApp {
    window: Window,
    pass: Pass,
    main_view: View,
    quad: ShaderQuadIns,
    quad_area: Area,
    knob_ids: [ComponentId; 3],
    knob_drag_offset: [Vec2; 3],
    #[cfg(not(feature = "disable-interaction"))]
    folded: bool,
    #[cfg(not(feature = "disable-interaction"))]
    splitter: Splitter,
    #[cfg(not(feature = "disable-interaction"))]
    text_editor: TextEditor,
    #[cfg(not(feature = "disable-interaction"))]
    text_buffer: TextBuffer,
    #[cfg(not(feature = "disable-interaction"))]
    error_message: String,
}

impl LogoApp {
    fn new(_cx: &mut Cx) -> Self {
        Self {
            window: Window { create_inner_size: Some(vec2(1200., 600.)), ..Window::default() },
            pass: Pass::default(),
            main_view: View::default(),
            quad: ShaderQuadIns { knobs: [vec2(0.2, 0.55), vec2(0.6, 0.05), vec2(0.5, 0.45)], ..Default::default() },
            quad_area: Area::Empty,
            knob_ids: Default::default(),
            knob_drag_offset: Default::default(),
            #[cfg(not(feature = "disable-interaction"))]
            folded: false,
            #[cfg(not(feature = "disable-interaction"))]
            splitter: Splitter {
                pos: 540.,
                align: SplitterAlign::Last,
                _hit_state_margin: Some(Padding::vh(0., 10.)),
                ..Splitter::default()
            },
            #[cfg(not(feature = "disable-interaction"))]
            text_editor: TextEditor { folding_depth: 2, folded_font_scale: 0.4, top_padding: 5., ..TextEditor::default() },
            #[cfg(not(feature = "disable-interaction"))]
            text_buffer: TextBuffer::from_utf8(CODE.code()),
            #[cfg(not(feature = "disable-interaction"))]
            error_message: "".to_string(),
        }
    }

    fn is_in_bounding_box(&self, pos: Vec2) -> bool {
        let mut bounding_box_min = self.quad.knobs[0];
        let mut bounding_box_max = self.quad.knobs[0];
        for knob in self.quad.knobs.iter().chain(self.quad.knobs_mirror.iter()) {
            bounding_box_min = bounding_box_min.min(knob);
            bounding_box_max = bounding_box_max.max(knob);
        }
        let rect = Rect {
            pos: bounding_box_min * self.quad.base.rect_size,
            size: (bounding_box_max - bounding_box_min) * self.quad.base.rect_size,
        };
        rect.add_padding(Padding::all(KNOB_RADIUS * 2.)).contains(pos)
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        #[cfg(not(feature = "disable-interaction"))]
        {
            if let SplitterEvent::Moving { new_pos: _ } = self.splitter.handle(cx, event) {
                cx.request_draw();
            }
            let ce = self.text_editor.handle(cx, event, &mut self.text_buffer);
            let code_fragments = [
                Cx::STD_SHADER,
                QuadIns::SHADER,
                INSTANCE_CODE,
                CodeFragment::Dynamic { name: "".to_string(), code: self.text_buffer.get_as_string() },
            ];
            match ce {
                TextEditorEvent::Change => {
                    if let Err(err) = SHADER.update(cx, &code_fragments) {
                        self.error_message = err.format_for_console(&code_fragments);
                    } else {
                        self.error_message = "".to_string();
                    }
                    cx.request_draw();
                }
                _ => {}
            }
        }

        #[cfg(not(feature = "disable-interaction"))]
        {
            match event {
                Event::Construct => {
                    self.text_editor.start_code_folding(cx, &self.text_buffer);
                    self.folded = true;
                }
                Event::PointerHover(fh) => {
                    if self.is_in_bounding_box(fh.abs) {
                        self.quad.show_knobs = 1.0;
                    } else {
                        self.quad.show_knobs = 0.0;
                    }
                    if let Some(rect) = self.text_editor.view.area().get_rect_for_first_instance(cx) {
                        if rect.contains(fh.abs) {
                            if self.folded {
                                self.text_editor.start_code_unfolding(cx, &self.text_buffer);
                                self.folded = false;
                            }
                        } else if !self.folded {
                            self.text_editor.start_code_folding(cx, &self.text_buffer);
                            self.folded = true;
                        }
                    }
                }
                Event::PointerDown(pd) => {
                    if self.is_in_bounding_box(pd.abs) {
                        self.quad.show_knobs = 1.0;
                    } else {
                        self.quad.show_knobs = 0.0;
                    }
                }
                _ => {}
            }

            for (index, unmultiplied_pos) in self.quad.knobs.iter_mut().enumerate() {
                let pos = *unmultiplied_pos * self.quad.base.rect_size;
                let rect = Rect { pos, size: vec2(0., 0.) }.add_padding(Padding::all(KNOB_RADIUS));
                match event.hits_pointer(cx, self.knob_ids[index], Some(rect)) {
                    Event::PointerDown(pd) => {
                        self.knob_drag_offset[index] = pd.abs - pos;
                        self.quad.down[index] = 1.;
                    }
                    Event::PointerHover(_fh) => {
                        cx.set_hover_mouse_cursor(MouseCursor::Hand);
                    }
                    Event::PointerMove(pm) => {
                        *unmultiplied_pos = (pm.abs - self.knob_drag_offset[index]) / self.quad.base.rect_size;
                    }
                    Event::PointerUp(_) => {
                        self.quad.down[index] = 0.;
                    }
                    _ => (),
                }
            }
        }

        self.update_shader(cx);
        cx.request_next_frame();
    }

    fn update_shader(&mut self, cx: &mut Cx) {
        self.quad.knobs_mirror[1] =
            vec2(1. - self.quad.knobs[1].x, self.quad.knobs[0].y + self.quad.knobs[2].y - self.quad.knobs[1].y);
        self.quad.knobs_mirror[0] = vec2(1. - self.quad.knobs[0].x, self.quad.knobs[2].y);
        self.quad.knobs_mirror[2] = vec2(1. - self.quad.knobs[2].x, self.quad.knobs[0].y);
        self.quad.time = ((cx.last_event_time / ANIMATION_LENGTH) % 1.0) as f32;
        *self.quad_area.get_first_mut(cx) = self.quad.clone();
    }

    fn draw_shader(&mut self, cx: &mut Cx) {
        #[cfg(not(feature = "disable-interaction"))]
        {
            cx.begin_padding_box(Padding::all(10.));
            cx.begin_bottom_box();
            TextIns::draw_walk(
                cx,
                &self.error_message,
                &TextInsProps {
                    text_style: TextStyle { font_size: 14., ..TEXT_STYLE_MONO },
                    wrapping: Wrapping::Word,
                    ..TextInsProps::default()
                },
            );
            cx.end_bottom_box();
            cx.end_padding_box();
        }
        self.quad.base = QuadIns::from_rect(cx.get_box_rect()).with_draw_depth(1.0);
        self.quad_area = cx.add_instances(&SHADER, &[self.quad.clone()]);
        self.update_shader(cx);
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("#000"));
        self.main_view.begin_view(cx, LayoutSize::FILL);

        #[cfg(feature = "disable-interaction")]
        {
            self.draw_shader(cx);
        }
        #[cfg(not(feature = "disable-interaction"))]
        {
            self.splitter.begin_draw(cx);
            {
                self.draw_shader(cx);
            }
            self.splitter.mid_draw(cx);
            {
                Self::update_token_chunks(&mut self.text_buffer);
                self.text_editor.begin_text_editor(cx, &self.text_buffer, None);
                for (index, token_chunk) in self.text_buffer.token_chunks.iter_mut().enumerate() {
                    self.text_editor.draw_chunk(cx, index, &self.text_buffer.flat_text, token_chunk, &self.text_buffer.markers);
                }
                self.text_editor.end_text_editor(cx, &self.text_buffer);
            }
            self.splitter.end_draw(cx);
        }

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }

    fn update_token_chunks(text_buffer: &mut TextBuffer) {
        if text_buffer.needs_token_chunks() && !text_buffer.lines.is_empty() {
            let mut state = TokenizerState::new(&text_buffer.lines);
            let mut tokenizer = MprsTokenizer::default();
            let mut pair_stack = Vec::new();
            loop {
                let offset = text_buffer.flat_text.len();
                let token_type = tokenizer.next_token(&mut state, &mut text_buffer.flat_text, &text_buffer.token_chunks);
                if TokenChunk::push_with_pairing(
                    &mut text_buffer.token_chunks,
                    &mut pair_stack,
                    state.next,
                    offset,
                    text_buffer.flat_text.len(),
                    token_type,
                ) {
                    text_buffer.was_invalid_pair = true;
                }

                if token_type == TokenType::Eof {
                    break;
                }
            }
            if !pair_stack.is_empty() {
                text_buffer.was_invalid_pair = true;
            }
        }
    }
}

main_app!(LogoApp);
