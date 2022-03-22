use crate::background::*;
use crate::scrollshadow::*;
use crate::scrollview::*;
use crate::textbuffer::*;
use crate::textcursor::*;
use crate::tokentype::*;
use zaplib::*;

static SHADER_INDENT_LINES: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            uniform indent_sel: float;
            instance color: vec4;
            instance indent_id: float;
            fn pixel() -> vec4 {
                let col = color;
                let thickness = 0.8 + dpi_dilate * 0.5;
                if indent_id == indent_sel {
                    col *= vec4(1., 1., 1., 1.);
                    thickness *= 1.3;
                }
                else {
                    col *= vec4(0.75, 0.75, 0.75, 0.75);
                }
                let df = Df::viewport(pos * rect_size);
                df.move_to(vec2(1., -1.));
                df.line_to(vec2(1., rect_size.y + 1.));
                return df.stroke(col, thickness);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};
static SHADER_CURSOR: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            uniform blink: float;
            instance color: vec4;
            fn pixel() -> vec4 {
                if blink<0.5 {
                    return vec4(color.rgb * color.a, color.a);
                }
                else {
                    return vec4(0., 0., 0., 0.);
                }
            }"#
        ),
    ],
    ..Shader::DEFAULT
};
static SHADER_SELECTION: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance color: vec4;
            instance prev_x: float;
            instance prev_w: float;
            instance next_x: float;
            instance next_w: float;

            const gloopiness: float = 8.;
            const border_radius: float = 2.;

            impl Df {
                fn gloop(inout self, k: float) {
                    let h = clamp(0.5 + 0.5 * (self.old_shape - self.field) / k, 0.0, 1.0);
                    self.old_shape = self.shape = mix(self.old_shape, self.field, h) - k * h * (1.0 - h);
                }
            }

            fn vertex() -> vec4 { // custom vertex shader because we widen the draweable area a bit for the gloopiness
                let shift: vec2 = -draw_scroll;
                let clipped: vec2 = clamp(
                    geom * vec2(rect_size.x + 16., rect_size.y) + rect_pos + shift - vec2(8., 0.),
                    draw_clip.xy,
                    draw_clip.zw
                );
                pos = (clipped - shift - rect_pos) / rect_size;
                return camera_projection * (camera_view *
                    vec4(clipped.x, clipped.y, draw_depth + draw_zbias, 1.));
            }

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);
                df.box(vec2(0.), rect_size, border_radius);
                if prev_w > 0. {
                    df.box(vec2(prev_x, -rect_size.y), vec2(prev_w, rect_size.y), border_radius);
                    df.gloop(gloopiness);
                }
                if next_w > 0. {
                    df.box(vec2(next_x, rect_size.y), vec2(next_w, rect_size.y), border_radius);
                    df.gloop(gloopiness);
                }
                //df_shape *= cos(pos.x*8.)+cos(pos.y*16.);
                return df.fill(color);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};
static SHADER_PAREN_PAIR: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance color: vec4;
            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);
                df.rect(vec2(0., rect_size.y - 1.5 - dpi_dilate), vec2(rect_size.x, 1.5 + dpi_dilate));
                return df.fill(color);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};
static SHADER_SEARCH_MARKER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance color: vec4;
            fn pixel() -> vec4 {
                let pos2 = vec2(pos.x, pos.y + 0.03 * sin(pos.x * rect_size.x));
                let df = Df::viewport(pos2 * rect_size);
                df.move_to(vec2(0., rect_size.y - 1.));
                df.line_to(rect_size.x, rect_size.y - 1.);
                return df.stroke(vec4(171.0/255.0,99.0/255.0,99.0/255.0,1.0), 0.8);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};
static SHADER_MESSAGE_MARKER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance color: vec4;
            fn pixel() -> vec4 {
                let pos2 = vec2(pos.x, pos.y + 0.03 * sin(pos.x * rect_size.x));
                let df = Df::viewport(pos2 * rect_size);
                df.move_to(vec2(0., rect_size.y - 1.));
                df.line_to(rect_size.x, rect_size.y - 1.);
                return df.stroke(color, 0.8);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

/// Convenient type of [`QuadIns`] which has a single `color` field, which is
/// drawn as the background by default. You pass in your own [`Shader`].
///
/// Currently only used within [`TextEditor`].
#[derive(Clone, Copy)]
#[repr(C)]
struct ColorBackgroundIns {
    quad: QuadIns,
    color: Vec4,
}
pub struct ColorBackground {
    shader: &'static Shader,
    color: Vec4,
}
impl ColorBackground {
    fn new(shader: &'static Shader) -> Self {
        Self { shader, color: Default::default() }
    }

    fn draw_quad_abs(&self, cx: &mut Cx, rect: Rect) {
        cx.add_instances(self.shader, &[ColorBackgroundIns { quad: QuadIns::from_rect(rect), color: self.color }]);
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct IndentLinesIns {
    quad: QuadIns,
    color: Vec4,
    indent_id: f32,
}
pub struct IndentLines {
    indent_sel: f32,
    area: Area,
    instances: Vec<IndentLinesIns>,
}
impl IndentLines {
    fn new() -> Self {
        Self { area: Area::Empty, indent_sel: Default::default(), instances: Default::default() }
    }
    fn write_uniforms(&self, cx: &mut Cx) {
        self.area.write_user_uniforms(cx, self.indent_sel);
    }
    fn set_indent_sel(&mut self, cx: &mut Cx, v: f32) {
        self.indent_sel = v;
        self.write_uniforms(cx);
    }
    fn clear(&mut self) {
        self.instances.clear();
    }
    fn make_quad_rel(&mut self, cx: &mut Cx, rect: Rect, color: Vec4, indent_id: f32) {
        self.instances.push(IndentLinesIns { quad: QuadIns::from_rect(rect.translate(cx.get_box_origin())), color, indent_id });
        self.write_uniforms(cx)
    }
    fn draw(&mut self, cx: &mut Cx) {
        self.area = cx.add_instances(&SHADER_INDENT_LINES, &self.instances);
        self.write_uniforms(cx)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CursorIns {
    base: QuadIns,
    color: Vec4,
}
pub struct Cursor {
    area: Area,
    blink: f32,
}
impl Cursor {
    fn new() -> Self {
        Self { area: Area::Empty, blink: 0.0 }
    }
    fn write_uniforms(&self, cx: &mut Cx) {
        self.area.write_user_uniforms(cx, self.blink);
    }
    fn set_blink(&mut self, cx: &mut Cx, v: f32) {
        self.blink = v;
        self.write_uniforms(cx);
    }
    fn draw_quad_rel(&mut self, cx: &mut Cx, rect: Rect) {
        self.area = cx.add_instances(
            &SHADER_CURSOR,
            &[CursorIns {
                base: QuadIns::from_rect(rect.translate(cx.get_box_origin())).with_draw_depth(1.3),
                color: COLOR_CURSOR,
            }],
        );
        self.write_uniforms(cx)
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct SelectionIns {
    base: QuadIns,
    color: Vec4,
    prev_x: f32,
    prev_w: f32,
    next_x: f32,
    next_w: f32,
}

pub struct TextEditor {
    pub component_id: ComponentId,
    pub view: ScrollView,
    pub gutter_bg: Background,
    pub cursor: Cursor,
    pub cursor_row: Background,
    pub paren_pair: ColorBackground,
    pub indent_lines: IndentLines,
    pub message_marker: ColorBackground,
    pub search_marker: ColorBackground,
    pub cursors: TextCursorSet,

    pub text_area: Area,
    pub text_glyphs: Vec<TextIns>,
    pub current_font_scale: f32,
    pub open_font_scale: f32,
    pub folded_font_scale: f32,
    pub line_number_width: f32,
    pub line_number_click_margin: f32,
    pub draw_line_numbers: bool,
    pub top_padding: f32,
    pub cursor_blink_speed: f64,
    pub _undo_id: u64,
    pub highlight_area_on: bool,

    pub mark_unmatched_parens: bool,
    pub draw_cursor_row: bool,
    pub search_markers_bypass: Vec<TextCursor>,
    pub folding_depth: usize,
    pub colors: CodeEditorColors,

    pub read_only: bool,
    pub multiline: bool,

    pub line_number_offset: usize,

    pub _scroll_pos_on_load: Option<Vec2>,
    pub _set_key_focus_on_load: bool,
    pub _set_last_cursor: Option<((usize, usize), bool)>,

    pub _line_number_chunk: Vec<char>,

    pub _scroll_pos: Vec2,
    pub _last_pointer_move: Option<Vec2>,
    pub _paren_stack: Vec<ParenItem>,
    pub _indent_stack: Vec<(Vec4, f32)>,
    pub _indent_id_alloc: f32,
    pub _indent_line_inst: Area,
    pub _bg_inst: Option<InstanceRangeArea>,
    pub _last_indent_color: Vec4,

    pub _line_geometry: Vec<LineGeom>,
    pub _anim_select: Vec<AnimSelect>,
    pub _visible_lines: usize,

    pub _select_scroll: Option<SelectScroll>,
    pub _grid_select_corner: Option<TextPos>,
    pub _is_row_select: bool,

    pub _last_cursor_pos: TextPos,

    pub _anim_font_scale: f32,
    pub _line_largest_font: f32,
    pub _anim_folding: AnimFolding,

    pub _monospace_size: Vec2,
    pub _monospace_base: Vec2,

    pub _tokens_on_line: usize,
    pub _line_was_folded: bool,
    pub _final_fill_height: f32,
    pub _draw_cursors: DrawCursors,
    pub _draw_search: DrawCursors,
    pub _draw_messages: DrawCursors,

    pub _cursor_blink_timer: Timer,
    pub _cursor_blink_flipflop: f32,
    pub _highlight_visibility: f32,

    pub _last_tabs: usize,
    pub _newline_tabs: usize,

    pub _last_lag_mutation_id: u32,

    pub _line_number_glyphs: Vec<TextIns>,
}

#[derive(Clone, PartialEq)]
pub enum TextEditorEvent {
    None,
    CursorMove,
    LagChange,
    Change,
    KeyFocus,
    KeyFocusLost,
    Escape,
    Return,
    Search(String),
    Decl(String),
}

#[derive(Default, Clone)]
pub struct CodeEditorColors {
    indent_line_unknown: Vec4,
    indent_line_fn: Vec4,
    indent_line_typedef: Vec4,
    indent_line_looping: Vec4,
    indent_line_flow: Vec4,
    paren_pair_match: Vec4,
    paren_pair_fail: Vec4,
    message_marker_error: Vec4,
    message_marker_warning: Vec4,
    message_marker_log: Vec4,
    line_number_normal: Vec4,
    line_number_highlight: Vec4,
    whitespace: Vec4,
    keyword: Vec4,
    flow: Vec4,
    looping: Vec4,
    identifier: Vec4,
    call: Vec4,
    type_name: Vec4,
    theme_name: Vec4,
    string: Vec4,
    number: Vec4,
    comment: Vec4,
    doc_comment: Vec4,
    paren_d1: Vec4,
    paren_d2: Vec4,
    operator: Vec4,
    delimiter: Vec4,
    unexpected: Vec4,
    warning: Vec4,
    error: Vec4,
    defocus: Vec4,
}

// TODO(JP): Make these constant Vec4's instead of recomputing them all the time.
const COLOR_GUTTER_BG: Vec4 = vec4(30.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0, 1.0);
const COLOR_INDENT_LINE_UNKNOWN: Vec4 = vec4(85.0 / 255.0, 85.0 / 255.0, 85.0 / 255.0, 1.0);
const COLOR_INDENT_LINE_FN: Vec4 = vec4(220.0 / 255.0, 220.0 / 255.0, 174.0 / 255.0, 1.0);
const COLOR_INDENT_LINE_TYPEDEF: Vec4 = vec4(91.0 / 255.0, 155.0 / 255.0, 211.0 / 255.0, 1.0);
const COLOR_INDENT_LINE_LOOPING: Vec4 = vec4(1.0, 140.0 / 255.0, 0.0 / 255.0, 1.0);
const COLOR_INDENT_LINE_FLOW: Vec4 = vec4(196.0 / 255.0, 133.0 / 255.0, 190.0 / 255.0, 1.0);
const COLOR_SELECTION: Vec4 = vec4(41.0 / 255.0, 78.0 / 255.0, 117.0 / 255.0, 1.0);
const COLOR_SELECTION_DEFOCUS: Vec4 = vec4(75.0 / 255.0, 75.0 / 255.0, 75.0 / 255.0, 1.0);
const COLOR_CURSOR: Vec4 = vec4(176.0 / 255.0, 176.0 / 255.0, 176.0 / 255.0, 1.0);
const COLOR_CURSOR_ROW: Vec4 = vec4(45.0 / 255.0, 45.0 / 255.0, 45.0 / 255.0, 1.0);
const COLOR_PAREN_PAIR_MATCH: Vec4 = vec4(1.0, 1.0, 1.0, 1.0);
const COLOR_PAREN_PAIR_FAIL: Vec4 = vec4(1.0, 0.0 / 255.0, 0.0 / 255.0, 1.0);
const COLOR_MESSAGE_MARKER_ERROR: Vec4 = vec4(200.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 1.0);
const COLOR_MESSAGE_MARKER_WARNING: Vec4 = vec4(0.0 / 255.0, 200.0 / 255.0, 0.0 / 255.0, 1.0);
const COLOR_MESSAGE_MARKER_LOG: Vec4 = vec4(200.0 / 255.0, 200.0 / 255.0, 200.0 / 255.0, 1.0);
const COLOR_SEARCH_MARKER: Vec4 = vec4(128.0 / 255.0, 64.0 / 255.0, 0.0 / 255.0, 1.0);
const COLOR_LINE_NUMBER_NORMAL: Vec4 = vec4(136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0, 1.0);
const COLOR_LINE_NUMBER_HIGHLIGHT: Vec4 = vec4(212.0 / 255.0, 212.0 / 255.0, 212.0 / 255.0, 1.0);
const COLOR_WHITESPACE: Vec4 = vec4(110.0 / 255.0, 110.0 / 255.0, 110.0 / 255.0, 1.0);
const COLOR_KEYWORD: Vec4 = vec4(91.0 / 255.0, 155.0 / 255.0, 211.0 / 255.0, 1.0);
const COLOR_FLOW: Vec4 = vec4(196.0 / 255.0, 133.0 / 255.0, 190.0 / 255.0, 1.0);
const COLOR_LOOPING: Vec4 = vec4(1.0, 140.0 / 255.0, 0.0 / 255.0, 1.0);
const COLOR_IDENTIFIER: Vec4 = vec4(212.0 / 255.0, 212.0 / 255.0, 212.0 / 255.0, 1.0);
const COLOR_CALL: Vec4 = vec4(220.0 / 255.0, 220.0 / 255.0, 174.0 / 255.0, 1.0);
const COLOR_TYPE_NAME: Vec4 = vec4(86.0 / 255.0, 201.0 / 255.0, 177.0 / 255.0, 1.0);
const COLOR_THEME_NAME: Vec4 = vec4(204.0 / 255.0, 145.0 / 255.0, 123.0 / 255.0, 1.0);
const COLOR_STRING: Vec4 = vec4(204.0 / 255.0, 145.0 / 255.0, 123.0 / 255.0, 1.0);
const COLOR_NUMBER: Vec4 = vec4(182.0 / 255.0, 206.0 / 255.0, 170.0 / 255.0, 1.0);
const COLOR_COMMENT: Vec4 = vec4(99.0 / 255.0, 141.0 / 255.0, 84.0 / 255.0, 1.0);
const COLOR_DOC_COMMENT: Vec4 = vec4(120.0 / 255.0, 171.0 / 255.0, 104.0 / 255.0, 1.0);
const COLOR_PAREN_D1: Vec4 = vec4(212.0 / 255.0, 212.0 / 255.0, 212.0 / 255.0, 1.0);
const COLOR_PAREN_D2: Vec4 = vec4(212.0 / 255.0, 212.0 / 255.0, 212.0 / 255.0, 1.0);
const COLOR_OPERATOR: Vec4 = vec4(212.0 / 255.0, 212.0 / 255.0, 212.0 / 255.0, 1.0);
const COLOR_DELIMITER: Vec4 = vec4(212.0 / 255.0, 212.0 / 255.0, 212.0 / 255.0, 1.0);
const COLOR_UNEXPECTED: Vec4 = vec4(1.0, 0.0 / 255.0, 0.0 / 255.0, 1.0);
const COLOR_WARNING: Vec4 = vec4(225.0 / 255.0, 229.0 / 255.0, 112.0 / 255.0, 1.0);
const COLOR_ERROR: Vec4 = vec4(254.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 1.0);
const COLOR_DEFOCUS: Vec4 = vec4(128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 1.0);

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            component_id: Default::default(),
            read_only: false,
            multiline: true,
            cursors: TextCursorSet::default(),

            indent_lines: IndentLines::new(),

            view: ScrollView::new_standard_vh(),

            gutter_bg: Background::default().with_draw_depth(1.4),

            colors: CodeEditorColors::default(),

            cursor: Cursor::new(),
            cursor_row: Background::default(),
            paren_pair: ColorBackground::new(&SHADER_PAREN_PAIR),
            message_marker: ColorBackground::new(&SHADER_MESSAGE_MARKER),
            search_marker: ColorBackground::new(&SHADER_SEARCH_MARKER),
            text_area: Area::Empty,
            text_glyphs: vec![],
            current_font_scale: 1.0,
            open_font_scale: 1.0,
            folded_font_scale: 0.07,
            line_number_width: 45.,
            line_number_click_margin: 10.,
            draw_line_numbers: true,
            cursor_blink_speed: 0.5,
            top_padding: 27.,
            mark_unmatched_parens: true,
            highlight_area_on: true,
            draw_cursor_row: true,
            line_number_offset: 0,
            search_markers_bypass: Vec::new(),
            _scroll_pos_on_load: None,
            _set_key_focus_on_load: false,
            _set_last_cursor: None,
            _monospace_size: Vec2::default(),
            _monospace_base: Vec2::default(),
            _last_pointer_move: None,
            _tokens_on_line: 0,
            _line_was_folded: false,
            _scroll_pos: Vec2::default(),
            _visible_lines: 0,
            _undo_id: 0,

            _line_geometry: Vec::new(),

            _anim_select: Vec::new(),
            _grid_select_corner: None,
            _is_row_select: false,
            _highlight_visibility: 0.,
            _bg_inst: None,
            _line_number_chunk: Vec::new(),

            _anim_font_scale: 1.0,
            _line_largest_font: 0.,
            _final_fill_height: 0.,
            folding_depth: 2,
            _anim_folding: AnimFolding { state: AnimFoldingState::Open, focussed_line: 0, did_animate: false },
            _select_scroll: None,
            _draw_cursors: DrawCursors::default(),
            _draw_search: DrawCursors::default(),
            _draw_messages: DrawCursors::default(),

            _paren_stack: Vec::new(),
            _indent_stack: Vec::new(),
            _indent_id_alloc: 0.0,
            _indent_line_inst: Area::Empty,

            _last_cursor_pos: TextPos::zero(),
            _last_indent_color: Vec4::default(),

            _cursor_blink_timer: Timer::empty(),
            _cursor_blink_flipflop: 0.,
            _last_lag_mutation_id: 0,
            _last_tabs: 0,
            _newline_tabs: 0,

            _line_number_glyphs: Vec::new(),
        }
    }
}
impl TextEditor {
    pub fn apply_style(&mut self) {
        // copy over colors
        self.colors.indent_line_unknown = COLOR_INDENT_LINE_UNKNOWN;
        self.colors.indent_line_fn = COLOR_INDENT_LINE_FN;
        self.colors.indent_line_typedef = COLOR_INDENT_LINE_TYPEDEF;
        self.colors.indent_line_looping = COLOR_INDENT_LINE_LOOPING;
        self.colors.indent_line_flow = COLOR_INDENT_LINE_FLOW;
        self.search_marker.color = COLOR_SEARCH_MARKER;
        self.colors.paren_pair_match = COLOR_PAREN_PAIR_MATCH;
        self.colors.paren_pair_fail = COLOR_PAREN_PAIR_FAIL;
        self.colors.message_marker_error = COLOR_MESSAGE_MARKER_ERROR;
        self.colors.message_marker_warning = COLOR_MESSAGE_MARKER_WARNING;
        self.colors.message_marker_log = COLOR_MESSAGE_MARKER_LOG;
        self.colors.line_number_normal = COLOR_LINE_NUMBER_NORMAL;
        self.colors.line_number_highlight = COLOR_LINE_NUMBER_HIGHLIGHT;
        self.colors.whitespace = COLOR_WHITESPACE;
        self.colors.keyword = COLOR_KEYWORD;
        self.colors.flow = COLOR_FLOW;
        self.colors.looping = COLOR_LOOPING;
        self.colors.identifier = COLOR_IDENTIFIER;
        self.colors.call = COLOR_CALL;
        self.colors.type_name = COLOR_TYPE_NAME;
        self.colors.theme_name = COLOR_THEME_NAME;
        self.colors.string = COLOR_STRING;
        self.colors.number = COLOR_NUMBER;
        self.colors.comment = COLOR_COMMENT;
        self.colors.doc_comment = COLOR_DOC_COMMENT;
        self.colors.paren_d1 = COLOR_PAREN_D1;
        self.colors.paren_d2 = COLOR_PAREN_D2;
        self.colors.operator = COLOR_OPERATOR;
        self.colors.delimiter = COLOR_DELIMITER;
        self.colors.unexpected = COLOR_UNEXPECTED;
        self.colors.warning = COLOR_WARNING;
        self.colors.error = COLOR_ERROR;
        self.colors.defocus = COLOR_DEFOCUS;
    }

    fn reset_cursor_blinker(&mut self, cx: &mut Cx) {
        cx.stop_timer(&mut self._cursor_blink_timer);
        self._cursor_blink_timer = cx.start_timer(self.cursor_blink_speed * 0.5, false);
        self._cursor_blink_flipflop = 0.;
        self.cursor.set_blink(cx, self._cursor_blink_flipflop);
    }

    fn handle_pointer_down(&mut self, cx: &mut Cx, pe: &PointerDownEvent, text_buffer: &mut TextBuffer) {
        cx.set_down_mouse_cursor(MouseCursor::Text);
        // give us the focus
        self.set_key_focus(cx);
        self._undo_id += 1;
        let offset;
        if pe.rel.x < self.line_number_width - self.line_number_click_margin {
            offset = self.compute_offset_from_ypos(cx, pe.abs.y, text_buffer, false);
            let range = text_buffer.get_nearest_line_range(offset);
            self.cursors.set_last_clamp_range(range);
            self._is_row_select = true;
        } else {
            offset = if let Some(o) = TextIns::closest_offset(cx, &self.text_area, pe.abs, TEXT_STYLE_MONO.line_spacing) {
                o
            } else {
                return;
            };
            match pe.tap_count {
                1 => {}
                2 => {
                    if let Some((coffset, len)) = TextCursorSet::get_nearest_token_chunk(offset, text_buffer) {
                        self.cursors.set_last_clamp_range((coffset, len));
                    }
                }
                3 => {
                    if let Some((coffset, len)) = TextCursorSet::get_nearest_token_chunk(offset, text_buffer) {
                        //self.cursors.set_last_clamp_range((coffset, len));
                        let (start, line_len) = text_buffer.get_nearest_line_range(offset);
                        let mut chunk_offset = coffset;
                        let mut chunk_len = len;
                        if start < chunk_offset {
                            chunk_len += chunk_offset - start;
                            chunk_offset = start;
                            if line_len > chunk_len {
                                chunk_len = line_len;
                            }
                        }
                        self.cursors.set_last_clamp_range((chunk_offset, chunk_len));
                    } else {
                        let range = text_buffer.get_nearest_line_range(offset);
                        self.cursors.set_last_clamp_range(range);
                    }
                }
                _ => {
                    //let range = (0, text_buffer.calc_char_count());
                    //self.cursors.set_last_clamp_range(range);
                }
            }
            // ok so we should scan a range
        }

        if pe.modifiers.shift {
            if pe.modifiers.logo || pe.modifiers.control {
                // grid select
                let pos = self.compute_grid_text_pos_from_abs(cx, pe.abs);
                self._grid_select_corner = Some(self.cursors.grid_select_corner(pos, text_buffer));
                self.cursors.grid_select(self._grid_select_corner.unwrap(), pos, text_buffer);
                if self.cursors.set.is_empty() {
                    self.cursors.clear_and_set_last_cursor_head_and_tail(offset, offset, text_buffer);
                }
            } else {
                // simply place selection
                self.cursors.clear_and_set_last_cursor_head(offset, text_buffer);
            }
        } else {
            // cursor drag with possible add
            if pe.modifiers.logo || pe.modifiers.control {
                self.cursors.add_last_cursor_head_and_tail(offset, offset, text_buffer);
            } else {
                self.cursors.clear_and_set_last_cursor_head_and_tail(offset, offset, text_buffer);
            }
        }

        cx.request_draw();
        self._last_pointer_move = Some(pe.abs);
        //self.update_highlight(cx, text_buffer);
        self.reset_cursor_blinker(cx);
    }

    fn handle_pointer_move(&mut self, cx: &mut Cx, pe: &PointerMoveEvent, text_buffer: &mut TextBuffer) {
        let cursor_moved = if let Some(grid_select_corner) = self._grid_select_corner {
            let pos = self.compute_grid_text_pos_from_abs(cx, pe.abs);
            self.cursors.grid_select(grid_select_corner, pos, text_buffer)
        } else if self._is_row_select {
            let offset = self.compute_offset_from_ypos(cx, pe.abs.y, text_buffer, true);
            self.cursors.set_last_cursor_head(offset, text_buffer)
        } else if let Some(offset) = TextIns::closest_offset(cx, &self.text_area, pe.abs, TEXT_STYLE_MONO.line_spacing) {
            self.cursors.set_last_cursor_head(offset, text_buffer)
        } else {
            false
        };

        self._last_pointer_move = Some(pe.abs);
        // determine selection drag scroll dynamics
        let repaint_scroll = self.check_select_scroll_dynamics(pe);
        //if cursor_moved {
        //     self.update_highlight(cx, text_buffer);
        //};
        if repaint_scroll || cursor_moved {
            cx.request_draw();
        }
        if cursor_moved {
            self.reset_cursor_blinker(cx);
        }
    }

    fn handle_pointer_up(&mut self, cx: &mut Cx, _pe: &PointerUpEvent, _text_buffer: &mut TextBuffer) {
        self.cursors.clear_last_clamp_range();
        self._select_scroll = None;
        self._last_pointer_move = None;
        self._grid_select_corner = None;
        self._is_row_select = false;
        //self.update_highlight(cx, text_buffer);
        self.reset_cursor_blinker(cx);
    }

    fn handle_key_down(&mut self, cx: &mut Cx, ke: &KeyEvent, text_buffer: &mut TextBuffer) -> bool {
        let cursor_moved = match ke.key_code {
            KeyCode::KeyE => {
                if ke.modifiers.logo || ke.modifiers.control {
                    let pos = self.cursors.get_last_cursor_head();
                    let mut moved = false;
                    for result in text_buffer.markers.search_cursors.iter().rev() {
                        if result.head < pos {
                            if ke.modifiers.shift {
                                self.cursors.add_last_cursor_head_and_tail(result.head, result.tail, text_buffer);
                            } else {
                                self.cursors.set_last_cursor_head_and_tail(result.head, result.tail, text_buffer);
                            }
                            moved = true;
                            break;
                        }
                    }

                    moved
                } else {
                    false
                }
            }
            KeyCode::KeyD => {
                if ke.modifiers.logo || ke.modifiers.control {
                    let pos = self.cursors.get_last_cursor_head();
                    let mut moved = false;
                    for result in text_buffer.markers.search_cursors.iter() {
                        if result.tail > pos {
                            if ke.modifiers.shift {
                                self.cursors.add_last_cursor_head_and_tail(result.head, result.tail, text_buffer);
                            } else {
                                self.cursors.set_last_cursor_head_and_tail(result.head, result.tail, text_buffer);
                            }
                            moved = true;
                            break;
                        }
                    }
                    moved
                } else {
                    false
                }
            }
            KeyCode::ArrowUp => {
                if !self.multiline || ke.modifiers.logo || ke.modifiers.control {
                    false
                } else {
                    if self._anim_folding.state.is_folded() && self.cursors.set.len() == 1 {
                        // compute the nearest nonfolded line up
                        let delta = self.compute_next_unfolded_line_up(text_buffer);
                        self.cursors.move_up(delta, ke.modifiers.shift, text_buffer);
                    } else {
                        self.cursors.move_up(1, ke.modifiers.shift, text_buffer);
                    }
                    self._undo_id += 1;
                    true
                }
            }
            KeyCode::ArrowDown => {
                if !self.multiline || ke.modifiers.logo || ke.modifiers.control {
                    false
                } else {
                    if self._anim_folding.state.is_folded() && self.cursors.set.len() == 1 {
                        // compute the nearest nonfolded line down
                        let delta = self.compute_next_unfolded_line_down(text_buffer);
                        self.cursors.move_down(delta, ke.modifiers.shift, text_buffer);
                    } else {
                        self.cursors.move_down(1, ke.modifiers.shift, text_buffer);
                    }
                    self._undo_id += 1;
                    true
                }
            }
            KeyCode::ArrowLeft => {
                if ke.modifiers.logo || ke.modifiers.control {
                    // token skipping
                    self.cursors.move_left_nearest_token(ke.modifiers.shift, text_buffer)
                } else {
                    self.cursors.move_left(1, ke.modifiers.shift, text_buffer);
                }
                self._undo_id += 1;
                true
            }
            KeyCode::ArrowRight => {
                if ke.modifiers.logo || ke.modifiers.control {
                    // token skipping
                    self.cursors.move_right_nearest_token(ke.modifiers.shift, text_buffer)
                } else {
                    self.cursors.move_right(1, ke.modifiers.shift, text_buffer);
                }
                self._undo_id += 1;
                true
            }
            KeyCode::PageUp => {
                self.cursors.move_up(self._visible_lines.max(5) - 4, ke.modifiers.shift, text_buffer);
                self._undo_id += 1;
                true
            }
            KeyCode::PageDown => {
                self.cursors.move_down(self._visible_lines.max(5) - 4, ke.modifiers.shift, text_buffer);
                self._undo_id += 1;
                true
            }
            KeyCode::Home => {
                self.cursors.move_home(ke.modifiers.shift, text_buffer);
                self._undo_id += 1;
                true
            }
            KeyCode::End => {
                self.cursors.move_end(ke.modifiers.shift, text_buffer);
                self._undo_id += 1;
                true
            }
            KeyCode::Backspace => {
                if !self.read_only {
                    self.cursors.backspace(text_buffer, self._undo_id);
                    true
                } else {
                    false
                }
            }
            KeyCode::Delete => {
                if !self.read_only {
                    self.cursors.delete(text_buffer);
                    true
                } else {
                    false
                }
            }
            KeyCode::KeyZ => {
                if !self.read_only {
                    if ke.modifiers.logo || ke.modifiers.control {
                        if ke.modifiers.shift {
                            // redo
                            text_buffer.redo(true, &mut self.cursors);
                            true
                        } else {
                            // undo
                            text_buffer.undo(true, &mut self.cursors);
                            true
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            KeyCode::KeyX => {
                // cut, the actual copy comes from the TextCopy event from the platform layer
                if !self.read_only && (ke.modifiers.logo || ke.modifiers.control) {
                    // cut
                    self.cursors.replace_text("", text_buffer, None);
                    true
                } else {
                    false
                }
            }
            KeyCode::KeyA => {
                // select all
                if ke.modifiers.logo || ke.modifiers.control {
                    // cut
                    self.cursors.select_all(text_buffer);
                    // don't scroll!
                    cx.request_draw();
                    false
                } else {
                    false
                }
            }
            KeyCode::Alt => {
                // how do we find the center line of the view
                // its simply the top line
                self.start_code_folding(cx, text_buffer);
                false
                //return CodeEditorEvent::FoldStart
            }
            KeyCode::Tab => {
                if !self.read_only {
                    if ke.modifiers.shift {
                        self.cursors.remove_tab(text_buffer, 4);
                    } else {
                        self.cursors.insert_tab(text_buffer, "    ");
                    }
                    true
                } else {
                    false
                }
            }
            KeyCode::Return => {
                if !self.read_only && self.multiline {
                    if !ke.modifiers.control && !ke.modifiers.logo {
                        self.cursors.insert_newline_with_indent(text_buffer);
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        if cursor_moved {
            //self.update_highlight(cx, text_buffer);
            self.scroll_last_cursor_visible(cx, text_buffer, 0.);
            cx.request_draw();
            self.reset_cursor_blinker(cx);
        }
        cursor_moved
    }

    fn handle_text_input(&mut self, cx: &mut Cx, te: &TextInputEvent, text_buffer: &mut TextBuffer) {
        if te.replace_last {
            text_buffer.undo(false, &mut self.cursors);
        }

        if !te.was_paste && te.input.len() == 1 {
            match te.input.chars().next().unwrap() {
                '(' => {
                    self.cursors.insert_around("(", ")", text_buffer);
                }
                '[' => {
                    self.cursors.insert_around("[", "]", text_buffer);
                }
                '{' => {
                    self.cursors.insert_around("{", "}", text_buffer);
                }
                '"' => {
                    self.cursors.insert_around("\"", "\"", text_buffer);
                }
                ')' => {
                    self.cursors.overwrite_if_exists_or_deindent(")", 4, text_buffer);
                }
                ']' => {
                    self.cursors.overwrite_if_exists_or_deindent("]", 4, text_buffer);
                }
                '}' => {
                    self.cursors.overwrite_if_exists_or_deindent("}", 4, text_buffer);
                }
                _ => {
                    self.cursors.replace_text(&te.input, text_buffer, None);
                }
            }
            // lets insert a newline
        } else if !self.multiline {
            let replaced = te.input.replace('\n', "");
            self.cursors.replace_text(&replaced, text_buffer, None);
        } else {
            self.cursors.replace_text(&te.input, text_buffer, None);
        }
        //self.update_highlight(cx, text_buffer);
        self.scroll_last_cursor_visible(cx, text_buffer, 0.);
        cx.request_draw();
        self.reset_cursor_blinker(cx);

        cx.send_signal(text_buffer.signal, TextBuffer::STATUS_DATA_UPDATE);
    }

    pub fn handle_live_replace(
        &mut self,
        cx: &mut Cx,
        range: (usize, usize),
        what: &str,
        text_buffer: &mut TextBuffer,
        group: u64,
    ) -> bool {
        // let set the cursor selection
        self.cursors.clear_and_set_last_cursor_head_and_tail(range.1, range.0, text_buffer);
        self.cursors.replace_text(what, text_buffer, Some(TextUndoGrouping::LiveEdit(group)));
        self.scroll_last_cursor_visible(cx, text_buffer, 0.);
        cx.request_draw();
        self.reset_cursor_blinker(cx);
        /*
        // do inplace update so we don't need to re-tokenize possibly
        if what.len() == range.1 - range.0 {
            for (index, c) in what.chars().enumerate() {
                text_buffer.flat_text[range.0 + index] = c;
            }
            return true
        }*/
        false
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event, text_buffer: &mut TextBuffer) -> TextEditorEvent {
        if self.view.handle(cx, event) {
            if let Some(last_pointer_move) = self._last_pointer_move {
                if let Some(grid_select_corner) = self._grid_select_corner {
                    let pos = self.compute_grid_text_pos_from_abs(cx, last_pointer_move);
                    self.cursors.grid_select(grid_select_corner, pos, text_buffer);
                } else if let Some(offset) =
                    TextIns::closest_offset(cx, &self.text_area, last_pointer_move, TEXT_STYLE_MONO.line_spacing)
                {
                    self.cursors.set_last_cursor_head(offset, text_buffer);
                }
            }
            // the editor actually redraws on scroll, its because we don't actually
            // generate the entire file as GPU text-buffer just the visible area
            // in JS this wasn't possible performantly but in Rust its a breeze.
            cx.request_draw();
        }
        let last_mutation_id = text_buffer.mutation_id;
        // global events
        match event {
            Event::Timer(te) => {
                if self._cursor_blink_timer.is_timer(te) {
                    if self.has_key_focus(cx) {
                        self._cursor_blink_timer = cx.start_timer(self.cursor_blink_speed, false);
                    }
                    // update the cursor uniform to blink it.
                    self._cursor_blink_flipflop = 1.0 - self._cursor_blink_flipflop;
                    self._undo_id += 1;
                    self._highlight_visibility = 1.0;
                    self.cursor.set_blink(cx, self._cursor_blink_flipflop);

                    // ok see if we changed.
                    if self._last_lag_mutation_id != text_buffer.mutation_id {
                        let was_filechange = self._last_lag_mutation_id != 0;
                        self._last_lag_mutation_id = text_buffer.mutation_id;
                        if was_filechange {
                            // lets post a signal on the textbuffer
                            return TextEditorEvent::LagChange;
                        }
                    }
                }
            }
            Event::Signal(se) => {
                if let Some(statusses) = se.signals.get(&text_buffer.signal) {
                    for status in statusses {
                        if *status == TextBuffer::STATUS_MESSAGE_UPDATE
                            || *status == TextBuffer::STATUS_SEARCH_UPDATE
                            || *status == TextBuffer::STATUS_DATA_UPDATE
                        {
                            cx.request_draw();
                        } else if *status == TextBuffer::STATUS_KEYBOARD_UPDATE {
                            if let Some(KeyCode::Alt) = &text_buffer.keyboard.key_down {
                                self.start_code_folding(cx, text_buffer);
                            }
                            if let Some(KeyCode::Alt) = &text_buffer.keyboard.key_up {
                                self.start_code_unfolding(cx, text_buffer);
                            }
                        }
                    }
                }
            }
            _ => (),
        }
        let mut cursor_moved = false;
        // editor local
        match event.hits_pointer(cx, self.component_id, self.view.area().get_rect_for_first_instance(cx)) {
            Event::PointerDown(pe) => {
                self.handle_pointer_down(cx, &pe, text_buffer);
            }
            Event::PointerHover(_pe) => {
                cx.set_hover_mouse_cursor(MouseCursor::Text);
            }
            Event::PointerUp(pe) => {
                self.handle_pointer_up(cx, &pe, text_buffer);
            }
            Event::PointerMove(pe) => {
                self.handle_pointer_move(cx, &pe, text_buffer);
            }
            _ => (),
        };

        match event.hits_keyboard(cx, self.component_id) {
            Event::KeyFocus(_kf) => {
                self.reset_cursor_blinker(cx);
                cx.request_draw();
                return TextEditorEvent::KeyFocus;
            }
            Event::KeyFocusLost(_kf) => {
                cx.request_draw();
                return TextEditorEvent::KeyFocusLost;
            }
            Event::KeyDown(ke) => {
                if ke.key_code == KeyCode::Return && !self.read_only && !self.multiline {
                    return TextEditorEvent::Return;
                }
                if ke.key_code == KeyCode::Escape {
                    let pos = self.cursors.get_last_cursor_head();
                    self.cursors.clear_and_set_last_cursor_head_and_tail(pos, pos, text_buffer);
                    return TextEditorEvent::Escape;
                }
                if ke.key_code == KeyCode::KeyF && (ke.modifiers.logo || ke.modifiers.control) {
                    let search = self.cursors.get_ident_around_last_cursor_and_set(text_buffer);
                    return TextEditorEvent::Search(search);
                }
                if ke.key_code == KeyCode::KeyS && (ke.modifiers.logo || ke.modifiers.control) {
                    let search = self.cursors.get_ident_around_last_cursor_and_set(text_buffer);
                    return TextEditorEvent::Decl(search);
                }
                cursor_moved = self.handle_key_down(cx, &ke, text_buffer);
            }
            Event::KeyUp(ke) => {
                match ke.key_code {
                    KeyCode::Alt => {
                        self.start_code_unfolding(cx, text_buffer);
                    }
                    _ => (),
                }
                self.reset_cursor_blinker(cx);
            }
            Event::TextInput(te) => {
                if !self.read_only {
                    self.handle_text_input(cx, &te, text_buffer);
                }
            }
            Event::TextCopy => {
                cx.copy_text_to_clipboard(&self.cursors.get_all_as_string(text_buffer));
            }
            _ => (),
        }

        // i need to know if selection changed, ifso
        //
        if last_mutation_id != text_buffer.mutation_id {
            TextEditorEvent::Change
        } else if cursor_moved {
            TextEditorEvent::CursorMove
        } else {
            TextEditorEvent::None
        }
    }

    pub fn has_key_focus(&self, cx: &Cx) -> bool {
        cx.has_key_focus(Some(self.component_id))
    }

    pub fn set_key_focus(&mut self, cx: &mut Cx) {
        if self.view.area() == Area::Empty {
            self._set_key_focus_on_load = true;
            return;
        }
        cx.set_key_focus(Some(self.component_id));
        self.reset_cursor_blinker(cx);
    }

    pub fn begin_draw_objects(&mut self) {
        self.text_glyphs.clear();
        self.indent_lines.clear();
        self._line_number_glyphs.clear();
    }

    pub fn end_draw_objects(&mut self, cx: &mut Cx) {
        self.text_area = TextIns::draw_glyphs(
            cx,
            &self.text_glyphs,
            &DrawGlyphsProps { text_style: TEXT_STYLE_MONO, ..DrawGlyphsProps::default() },
        );
        self.indent_lines.draw(cx);
    }

    pub fn init_draw_state(&mut self, cx: &mut Cx, text_buffer: &TextBuffer) {
        self._monospace_base = TextIns::get_monospace_base(cx, &TEXT_STYLE_MONO);
        self.set_font_scale(cx, self.open_font_scale);
        self._draw_cursors = DrawCursors::default();
        self._draw_messages = DrawCursors::default();
        self._draw_search = DrawCursors::default();
        self._tokens_on_line = 0;
        self._visible_lines = 0;
        self._newline_tabs = 0;
        self._last_tabs = 0;
        self._indent_stack.truncate(0);
        self._indent_id_alloc = 1.0;
        self._paren_stack.truncate(0);
        self._draw_cursors.set_next(&self.cursors.set);
        self._draw_search.set_next(if !self.search_markers_bypass.is_empty() {
            &self.search_markers_bypass
        } else {
            &text_buffer.markers.search_cursors
        });
        self._line_geometry.truncate(0);
        self._line_largest_font = TEXT_STYLE_MONO.font_size;
        self._last_indent_color = self.colors.indent_line_unknown;
        // indent
        cx.move_draw_pos(self.line_number_width, self.top_padding);
    }

    pub fn begin_text_editor(&mut self, cx: &mut Cx, text_buffer: &TextBuffer, override_layout_size: Option<LayoutSize>) {
        // adjust dilation based on DPI factor

        if let Some(layout_size) = override_layout_size {
            self.view.begin_view(cx, layout_size);
            cx.begin_row(layout_size.width, layout_size.height);
        } else {
            self.view.begin_view(cx, LayoutSize::FILL);
            cx.begin_row(Width::Compute, Height::Compute);
        };

        self.apply_style();

        //println!("{:?}", self.cursors.set[0]);

        if self._set_key_focus_on_load {
            self._set_key_focus_on_load = false;
            self.set_key_focus(cx);
        }

        self.begin_draw_objects();

        if let Some(select_scroll) = &mut self._select_scroll {
            let scroll_pos = self.view.get_scroll_pos(cx);
            if self
                .view
                .set_scroll_pos(cx, Vec2 { x: scroll_pos.x + select_scroll.delta.x, y: scroll_pos.y + select_scroll.delta.y })
            {
                cx.request_draw();
            } else {
                select_scroll.at_end = true;
            }
        }

        if text_buffer.markers.mutation_id != text_buffer.mutation_id {
            self._draw_messages.term(&text_buffer.markers.message_cursors);
        } else {
            self._draw_messages.set_next(&text_buffer.markers.message_cursors);
        }
        self._last_cursor_pos = self.cursors.get_last_cursor_text_pos(text_buffer);

        // the TextCursor should automatically adjust in case a smaller TextBuffer is
        // applied.
        self.cursors.clamp_to_text_buffer(text_buffer);

        // lets compute our scroll line position and keep it where it is
        self.do_folding_animation_step(cx);

        self.init_draw_state(cx, text_buffer);

        self._scroll_pos = self.view.get_scroll_pos(cx);
    }

    fn do_folding_animation_step(&mut self, cx: &mut Cx) {
        // run the folding animation
        let anim_folding = &mut self._anim_folding;
        if anim_folding.state.is_animating() {
            anim_folding.state.next_anim_step();
            if anim_folding.state.is_animating() {
                cx.request_draw();
            }
            anim_folding.did_animate = true;
        } else {
            anim_folding.did_animate = false;
        }
        //let new_anim_font_size =
        self._anim_font_scale = anim_folding.state.get_font_size(self.open_font_scale, self.folded_font_scale);

        if self._anim_folding.did_animate {
            let mut ypos = self.top_padding;
            let mut ypos_at_line = ypos;
            let focus_line = self._anim_folding.focussed_line;
            if focus_line < self._line_geometry.len() {
                for (line, geom) in self._line_geometry.iter().enumerate() {
                    if focus_line == line {
                        ypos_at_line = ypos;
                    }
                    ypos += if geom.was_folded {
                        self._monospace_base.y * TEXT_STYLE_MONO.font_size * self._anim_font_scale
                    } else {
                        self._monospace_base.y * TEXT_STYLE_MONO.font_size
                    }
                }
                ypos += self._final_fill_height;
                let dy = self._line_geometry[focus_line].walk.y - ypos_at_line;
                let sv = self.view.get_scroll_view_total();
                self.view.set_scroll_view_total(cx, Vec2 { x: sv.x, y: ypos });
                let scroll_pos = self.view.get_scroll_pos(cx);
                self.view.set_scroll_pos(cx, Vec2 { x: scroll_pos.x, y: scroll_pos.y - dy });
            }
        }
    }

    fn line_is_visible(&self, cx: &mut Cx, min_height: f32, scroll: Vec2) -> bool {
        let pos = cx.get_draw_pos();
        let vy = cx.get_box_origin().y + scroll.y;
        let vh = cx.get_height_total();
        !(pos.y > vy + vh || pos.y + min_height < vy)
    }

    fn draw_new_line(&mut self, cx: &mut Cx) {
        // line geometry is used for scrolling look up of cursors
        let relative_offset = cx.get_draw_pos() - cx.get_box_origin();
        let line_geom = LineGeom {
            walk: relative_offset,
            font_size: self._line_largest_font,
            was_folded: self._line_was_folded,
            indent_id: if let Some((_, id)) = self._indent_stack.last() { *id } else { 0. },
        };

        // draw a linenumber if we are visible
        let origin = cx.get_box_origin();
        if self.draw_line_numbers && self.line_is_visible(cx, self._monospace_size.y, self._scroll_pos) {
            // lets format a number, we go to 4 numbers
            // yes this is dumb as rocks. but we need to be cheapnfast
            let mut line_number_text = String::with_capacity(6);

            let line_num = self._line_geometry.len() + 1 + self.line_number_offset;
            let mut scale = 10000;
            let mut fill = false;
            loop {
                let digit = ((line_num / scale) % 10) as u8;
                if digit != 0 {
                    fill = true;
                }
                if fill {
                    line_number_text.push((48 + digit) as char);
                } else {
                    line_number_text.push(' ');
                }
                if scale <= 1 {
                    break;
                }
                scale /= 10;
            }
            let draw_str_props = TextInsProps {
                wrapping: Wrapping::Line,
                text_style: TEXT_STYLE_MONO,
                font_scale: self.current_font_scale,
                draw_depth: 1.5,
                color: if line_num == self._last_cursor_pos.row + 1 {
                    self.colors.line_number_highlight
                } else {
                    self.colors.line_number_normal
                },
                ..TextInsProps::DEFAULT
            };
            let chunk_width = self._monospace_size.x * 5.0;
            self._line_number_glyphs.extend(TextIns::generate_2d_glyphs(
                &draw_str_props.text_style,
                &cx.fonts_data,
                cx.current_dpi_factor,
                draw_str_props.font_scale,
                draw_str_props.draw_depth,
                draw_str_props.color,
                vec2(
                    origin.x + (self.line_number_width - chunk_width - self.line_number_click_margin),
                    origin.y + line_geom.walk.y,
                ),
                0,
                &line_number_text.chars().collect::<Vec<char>>(),
                |_, _, _, _| 0.0,
            ));
        }

        cx.draw_new_line_min_height(self._monospace_size.y);

        cx.move_draw_pos(self.line_number_width, 0.);

        self._tokens_on_line = 0;
        //self._line_was_visible = false;

        self._draw_cursors.process_newline();
        self._draw_messages.process_newline();

        // search for all markings
        self._line_geometry.push(line_geom);
        self._line_largest_font = TEXT_STYLE_MONO.font_size;
    }

    fn draw_indent_lines(&mut self, cx: &mut Cx, geom_y: f32, tabs: usize) {
        let y_pos = geom_y - cx.get_box_origin().y;
        let tab_variable_width = self._monospace_base.x * 4. * TEXT_STYLE_MONO.font_size * self._anim_font_scale;
        let tab_fixed_width = self._monospace_base.x * 4. * TEXT_STYLE_MONO.font_size;
        let mut off = self.line_number_width;
        for i in 0..tabs {
            let (indent_color, indent_id) =
                if i < self._indent_stack.len() { self._indent_stack[i] } else { (self.colors.indent_line_unknown, 0.) };
            let tab_width = if i < self.folding_depth { tab_fixed_width } else { tab_variable_width };
            self.indent_lines.make_quad_rel(
                cx,
                Rect { pos: vec2(off, y_pos), size: vec2(tab_width, self._monospace_size.y) },
                indent_color,
                indent_id,
            );
            off += tab_width;
        }
    }

    pub fn draw_chunk(
        &mut self,
        cx: &mut Cx,
        token_chunks_index: usize,
        flat_text: &[char],
        token_chunk: &TokenChunk,
        markers: &TextBufferMarkers,
    ) {
        if token_chunk.len == 0 {
            return;
        }

        let token_type = token_chunk.token_type;
        let chunk = &flat_text[token_chunk.offset..(token_chunk.offset + token_chunk.len)]; //chunk;
        let offset = token_chunk.offset; // end_offset - chunk.len() - 1;
        let next_char = token_chunk.next;

        // maintain paren stack
        if token_type == TokenType::ParenOpen {
            self.draw_paren_open(token_chunks_index, offset, next_char, chunk);
        }

        // do indent depth walking
        if self._tokens_on_line == 0 {
            let font_scale = match token_type {
                TokenType::Whitespace => {
                    let tabs = chunk.len() >> 2;
                    while tabs > self._indent_stack.len() {
                        self._indent_stack.push((self._last_indent_color, self._indent_id_alloc));
                        // allocating an indent_id, we also need to
                        self._indent_id_alloc += 1.0;
                    }
                    while tabs < self._indent_stack.len() {
                        self._indent_stack.pop();
                        if let Some(indent) = self._indent_stack.last() {
                            self._last_indent_color = indent.0;
                        }
                    }
                    // lets do the code folding here. if we are tabs > fold line
                    // lets change the fontsize
                    if tabs >= self.folding_depth || next_char == '\n' {
                        // ok lets think. we need to move it over by the delta of 8 spaces * _anim_font_size
                        let dx = (self._monospace_base.x * TEXT_STYLE_MONO.font_size * 4. * (self.folding_depth as f32))
                            - (self._monospace_base.x
                                * TEXT_STYLE_MONO.font_size
                                * self._anim_font_scale
                                * 4.
                                * (self.folding_depth as f32));
                        cx.move_draw_pos(dx, 0.0);
                        self._line_was_folded = true;
                        self._anim_font_scale
                    } else {
                        self._line_was_folded = false;
                        self.open_font_scale
                    }
                }
                TokenType::Newline
                | TokenType::CommentLine
                | TokenType::CommentChunk
                | TokenType::CommentMultiBegin
                | TokenType::CommentMultiEnd
                | TokenType::Hash => {
                    self._line_was_folded = true;
                    self._anim_font_scale
                }
                _ => {
                    self._indent_stack.truncate(0);
                    self._line_was_folded = false;
                    self.open_font_scale
                }
            };
            self.set_font_scale(cx, font_scale);
        }
        // colorise indent lines properly
        if self._tokens_on_line < 4 {
            match token_type {
                TokenType::Flow => {
                    self._last_indent_color = self.colors.indent_line_flow;
                }
                TokenType::Looping => {
                    self._last_indent_color = self.colors.indent_line_looping;
                }
                TokenType::TypeDef => {
                    self._last_indent_color = self.colors.indent_line_typedef;
                }
                TokenType::Fn | TokenType::Call | TokenType::Macro => {
                    self._last_indent_color = self.colors.indent_line_fn;
                }
                _ => (),
            }
        }
        // lets check if the geom is visible
        if let Some(geom) = self.move_cursor_right_no_wrap(
            cx,
            self._monospace_size.x * (chunk.len() as f32),
            self._monospace_size.y,
            self._scroll_pos,
        ) {
            let mut mark_spaces = 0.0;
            // determine chunk color
            let color = match token_type {
                TokenType::Whitespace => {
                    if self._tokens_on_line == 0 && chunk[0] == ' ' {
                        let tabs = chunk.len() >> 2;
                        // if self._last_tabs
                        self._last_tabs = tabs;
                        self._newline_tabs = tabs;
                        self.draw_indent_lines(cx, geom.pos.y, tabs);
                    } else if next_char == '\n' {
                        mark_spaces = 1.0;
                    }
                    self.colors.whitespace
                }
                TokenType::Newline => {
                    if self._tokens_on_line == 0 {
                        self._newline_tabs = 0;
                        self.draw_indent_lines(cx, geom.pos.y, self._last_tabs);
                    } else {
                        self._last_tabs = self._newline_tabs;
                        self._newline_tabs = 0;
                    }
                    self.colors.whitespace
                }
                TokenType::BuiltinType => self.colors.keyword,
                TokenType::Keyword => self.colors.keyword,
                TokenType::Bool => self.colors.keyword,
                TokenType::Error => self.colors.error,
                TokenType::Warning => self.colors.warning,
                TokenType::Defocus => self.colors.defocus,
                TokenType::Flow => self.colors.flow,
                TokenType::Looping => self.colors.looping,
                TokenType::TypeDef => self.colors.keyword,
                TokenType::Impl => self.colors.keyword,
                TokenType::Fn => self.colors.keyword,
                TokenType::Identifier => self.colors.identifier,
                TokenType::Macro | TokenType::Call => self.colors.call,
                TokenType::TypeName => self.colors.type_name,
                TokenType::ThemeName => self.colors.theme_name,
                TokenType::Color => self.colors.string,
                TokenType::Regex => self.colors.string,
                TokenType::String => self.colors.string,
                TokenType::Number => self.colors.number,

                TokenType::StringMultiBegin => self.colors.string,
                TokenType::StringChunk => self.colors.string,
                TokenType::StringMultiEnd => self.colors.string,

                TokenType::CommentMultiBegin => self.colors.comment,
                TokenType::CommentMultiEnd => self.colors.comment,
                TokenType::CommentLine => self.colors.comment,
                TokenType::CommentChunk => self.colors.comment,
                TokenType::ParenOpen => {
                    let depth = self._paren_stack.len();
                    self._paren_stack.last_mut().unwrap().geom_open = Some(geom);
                    match depth % 2 {
                        0 => self.colors.paren_d1,
                        _ => self.colors.paren_d2,
                    }
                }
                TokenType::ParenClose => {
                    if let Some(paren) = self._paren_stack.last_mut() {
                        paren.geom_close = Some(geom);
                    } else if self.mark_unmatched_parens {
                        self.paren_pair.color = self.colors.paren_pair_fail;
                        self.paren_pair.draw_quad_abs(cx, geom);
                    }
                    let depth = self._paren_stack.len();
                    match depth % 2 {
                        0 => self.colors.paren_d1,
                        _ => self.colors.paren_d2,
                    }
                }
                TokenType::Operator => self.colors.operator,
                TokenType::Namespace => self.colors.operator,
                TokenType::Hash => self.colors.operator,
                TokenType::Delimiter => self.colors.delimiter,
                TokenType::Colon => self.colors.delimiter,
                TokenType::Splat => self.colors.operator,
                TokenType::Eof => self.colors.unexpected,
                TokenType::Unexpected => self.colors.unexpected,
            };

            if self._tokens_on_line == 0 {
                self._visible_lines += 1;
            }

            let cursors = &self.cursors.set;
            let last_cursor = self.cursors.last_cursor;
            let draw_cursors = &mut self._draw_cursors;
            let draw_messages = &mut self._draw_messages;
            let draw_search = &mut self._draw_search;

            let height = self._monospace_size.y;
            let search_cursors =
                if !self.search_markers_bypass.is_empty() { &self.search_markers_bypass } else { &markers.search_cursors };
            // actually generate the GPU data for the text
            let z = 2.0; // + self._paren_stack.len() as f32;
                         //self.text.z = z;
                         //let line_chunk = &mut self._line_chunk;
            if !search_cursors.is_empty() {
                // slow loop
                let char_callback = |ch, offset, x, w| {
                    //line_chunk.push((x, ch));
                    draw_search.mark_text_select_only(search_cursors, offset, x, geom.pos.y, w, height);
                    draw_messages.mark_text_select_only(&markers.message_cursors, offset, x, geom.pos.y, w, height);
                    draw_cursors.mark_text_with_cursor(cursors, ch, offset, x, geom.pos.y, w, height, z, last_cursor, mark_spaces)
                };
                self.text_glyphs.extend(TextIns::generate_2d_glyphs(
                    &TEXT_STYLE_MONO,
                    &cx.fonts_data,
                    cx.current_dpi_factor,
                    self.current_font_scale,
                    0.,
                    color,
                    geom.pos,
                    offset,
                    chunk,
                    char_callback,
                ));
            } else {
                let char_callback = |ch, offset, x, w| {
                    //line_chunk.push((x, ch));
                    draw_messages.mark_text_select_only(&markers.message_cursors, offset, x, geom.pos.y, w, height);
                    draw_cursors.mark_text_with_cursor(cursors, ch, offset, x, geom.pos.y, w, height, z, last_cursor, mark_spaces)
                };
                self.text_glyphs.extend(TextIns::generate_2d_glyphs(
                    &TEXT_STYLE_MONO,
                    &cx.fonts_data,
                    cx.current_dpi_factor,
                    self.current_font_scale,
                    0.,
                    color,
                    geom.pos,
                    offset,
                    chunk,
                    char_callback,
                ));
            };
        }
        self._tokens_on_line += 1;

        // Do all the Paren matching highlighting drawing
        if token_chunk.token_type == TokenType::ParenClose {
            self.draw_paren_close(cx, token_chunks_index, offset, next_char, chunk);
        } else if token_type == TokenType::Newline {
            self.draw_new_line(cx);
        }
    }

    fn move_cursor_right_no_wrap(&self, cx: &mut Cx, w: f32, h: f32, scroll: Vec2) -> Option<Rect> {
        // Save position before updating it
        let pos = cx.get_draw_pos();

        cx.add_box(LayoutSize::new(Width::Fix(w), Height::Fix(h)));
        let origin = cx.get_box_origin();
        let vx = origin.x + scroll.x;
        let vy = origin.y + scroll.y;
        let vw = cx.get_width_total();
        let vh = cx.get_height_total();
        if pos.x > vx + vw || pos.x + w < vx || pos.y > vy + vh || pos.y + h < vy {
            None
        } else {
            Some(Rect { pos: vec2(pos.x, pos.y), size: vec2(w, h) })
        }
    }

    fn draw_paren_open(&mut self, token_chunks_index: usize, offset: usize, next_char: char, chunk: &[char]) {
        let marked = if let Some(pos) = self.cursors.get_last_cursor_singular() {
            pos == offset || pos == offset + 1 && next_char != '(' && next_char != '{' && next_char != '['
        } else {
            false
        };

        self._paren_stack.push(ParenItem {
            pair_start: token_chunks_index, //self.token_chunks.len(),
            geom_open: None,
            geom_close: None,
            marked,
            exp_paren: chunk[0],
        });
    }

    fn draw_paren_close(&mut self, cx: &mut Cx, token_chunks_index: usize, offset: usize, next_char: char, chunk: &[char]) {
        //let token_chunks_len = self.token_chunks.len();
        if self._paren_stack.is_empty() {
            return;
        }
        let last = self._paren_stack.pop().unwrap();
        // !!!!self.token_chunks[last.pair_start].pair_token = token_chunks_index;
        if last.geom_open.is_none() && last.geom_close.is_none() {
            return;
        }
        if !self.has_key_focus(cx) {
            return;
        }
        if let Some(pos) = self.cursors.get_last_cursor_singular() {
            if self.mark_unmatched_parens {
                // cursor is near the last one or its marked
                let fail = if last.exp_paren == '(' && chunk[0] != ')'
                    || last.exp_paren == '[' && chunk[0] != ']'
                    || last.exp_paren == '{' && chunk[0] != '}'
                {
                    self.paren_pair.color = self.colors.paren_pair_fail;
                    true
                } else {
                    self.paren_pair.color = self.colors.paren_pair_match;
                    false
                };
                if fail
                    || pos == offset
                    || pos == offset + 1 && next_char != ')' && next_char != '}' && next_char != ']'
                    || last.marked
                {
                    // fuse the tokens
                    if last.pair_start + 1 == token_chunks_index && last.geom_open.is_some() && last.geom_close.is_some() {
                        let geom_open = last.geom_open.unwrap();
                        let geom_close = last.geom_open.unwrap();
                        let geom =
                            Rect { pos: geom_open.pos, size: vec2(geom_open.size.x + geom_close.size.x, geom_close.size.y) };
                        self.paren_pair.draw_quad_abs(cx, geom);
                    } else {
                        if let Some(rc) = last.geom_open {
                            self.paren_pair.draw_quad_abs(cx, rc);
                        }
                        if let Some(rc) = last.geom_close {
                            self.paren_pair.draw_quad_abs(cx, rc);
                        }
                    }
                }
            };
        }
    }

    fn draw_paren_unmatched(&mut self, cx: &mut Cx) {
        if !self.mark_unmatched_parens {
            return;
        }
        while !self._paren_stack.is_empty() {
            let last = self._paren_stack.pop().unwrap();
            if self.has_key_focus(cx) && last.geom_open.is_some() {
                self.paren_pair.color = self.colors.paren_pair_fail;
                if let Some(rc) = last.geom_open {
                    self.paren_pair.draw_quad_abs(cx, rc);
                }
            }
        }
    }

    pub fn end_text_editor(&mut self, cx: &mut Cx, text_buffer: &TextBuffer) {
        if self.multiline {
            // lets insert an empty newline at the bottom so its nicer to scroll
            self.draw_new_line(cx);
        }

        self.place_ime_and_draw_cursor_row(cx);
        self.draw_selections(cx);
        self.draw_message_markers(cx, text_buffer);
        self.draw_search_markers(cx);
        self.draw_paren_unmatched(cx);
        self.end_draw_objects(cx);
        self.draw_cursors(cx);
        if self.draw_line_numbers {
            self.gutter_bg.draw_with_scroll_sticky(
                cx,
                Rect { pos: cx.get_box_origin(), size: vec2(self.line_number_width, cx.get_height_total()) },
                COLOR_GUTTER_BG,
            );
        }
        TextIns::draw_glyphs_with_scroll_sticky(cx, &self._line_number_glyphs, &TEXT_STYLE_MONO, true, false);

        // inject a final page
        self._final_fill_height = cx.get_height_total() - self._monospace_size.y;
        self.draw_shadows(cx);

        // last bits
        self.do_selection_scrolling(cx, text_buffer);
        self.set_indent_line_highlight_id(cx);

        cx.end_row();
        self.view.end_view(cx);

        if let Some(((head, tail), at_top)) = self._set_last_cursor {
            self._set_last_cursor = None;
            self._scroll_pos_on_load = None;
            self.cursors.clear_and_set_last_cursor_head_and_tail(head, tail, text_buffer);
            // i want the thing to be the top
            if at_top {
                self.scroll_last_cursor_top(cx, text_buffer);
            } else {
                self.scroll_last_cursor_visible(cx, text_buffer, self._final_fill_height * 0.8);
            }

            cx.request_draw();
        } else if let Some(scroll_pos_on_load) = self._scroll_pos_on_load {
            self.view.set_scroll_pos(cx, scroll_pos_on_load);
            self._scroll_pos_on_load = None;
        }
    }

    pub fn set_last_cursor(&mut self, cx: &mut Cx, cursor: (usize, usize), at_top: bool) {
        self._set_last_cursor = Some((cursor, at_top));
        cx.request_draw();
    }

    fn draw_cursors(&mut self, cx: &mut Cx) {
        if self.has_key_focus(cx) {
            let origin = cx.get_box_origin();
            self.cursor.blink = self._cursor_blink_flipflop;
            for rc in &self._draw_cursors.cursors {
                self.cursor.draw_quad_rel(cx, Rect { pos: vec2(rc.x, rc.y) - origin, size: vec2(rc.w, rc.h) });
            }
        }
    }

    fn draw_shadows(&mut self, cx: &mut Cx) {
        ScrollShadow::draw_shadow_left_at(
            cx,
            Rect { pos: vec2(self.line_number_width, 0.), size: vec2(0., cx.get_height_total()) },
            1.0,
        );
        ScrollShadow::draw_shadow_top(cx, 1.0);
    }

    fn draw_message_markers(&mut self, cx: &mut Cx, text_buffer: &TextBuffer) {
        let origin = cx.get_box_origin();
        let message_markers = &mut self._draw_messages.selections;

        for mark in message_markers {
            let body = &text_buffer.markers.message_bodies[mark.index];
            self.message_marker.color = match body.level {
                TextBufferMessageLevel::Warning => self.colors.message_marker_warning,
                TextBufferMessageLevel::Error => self.colors.message_marker_error,
                TextBufferMessageLevel::Log => self.colors.message_marker_log,
            };
            self.message_marker.draw_quad_abs(cx, Rect { pos: cx.get_box_origin() + mark.rc.pos - origin, size: mark.rc.size });
        }
    }

    pub fn draw_search_markers(&mut self, cx: &mut Cx) {
        let origin = cx.get_box_origin();

        for mark in &self._draw_search.selections {
            self.search_marker.draw_quad_abs(cx, Rect { pos: cx.get_box_origin() + mark.rc.pos - origin, size: mark.rc.size });
        }
    }

    pub fn draw_selections(&mut self, cx: &mut Cx) {
        let color = if self.has_key_focus(cx) { COLOR_SELECTION } else { COLOR_SELECTION_DEFOCUS };
        let sel = &mut self._draw_cursors.selections;
        // draw selections
        let data: Vec<SelectionIns> = (0..sel.len())
            .map(|i| {
                let cur = &sel[i];
                let mut selection = SelectionIns {
                    base: QuadIns::from_rect(Rect { pos: cur.rc.pos, size: cur.rc.size }),
                    color,
                    ..SelectionIns::default()
                };
                // do we have a prev?
                if i > 0 && sel[i - 1].index == cur.index {
                    let p_rc = &sel[i - 1].rc;
                    selection.prev_x = p_rc.pos.x - cur.rc.pos.x;
                    selection.prev_w = p_rc.size.x;
                } else {
                    selection.prev_x = 0.;
                    selection.prev_w = -1.;
                }
                // do we have a next
                if i < sel.len() - 1 && sel[i + 1].index == cur.index {
                    let n_rc = &sel[i + 1].rc;
                    selection.next_x = n_rc.pos.x - cur.rc.pos.x;
                    selection.next_w = n_rc.size.x;
                } else {
                    selection.next_x = 0.;
                    selection.next_w = -1.;
                }
                selection
            })
            .collect();
        cx.add_instances(&SHADER_SELECTION, &data);
    }

    fn place_ime_and_draw_cursor_row(&mut self, cx: &mut Cx) {
        // place the IME
        if let Some(last_cursor) = self._draw_cursors.last_cursor {
            let rc = self._draw_cursors.cursors[last_cursor];
            if self.cursors.get_last_cursor_singular().is_some() {
                // lets draw the cursor line
                if self.draw_cursor_row {
                    self.cursor_row.draw(
                        cx,
                        Rect {
                            pos: vec2(self.line_number_width + cx.get_box_origin().x, rc.y),
                            size: vec2(cx.get_width_total().max(cx.get_box_bounds().x) - self.line_number_width, rc.h),
                        },
                        COLOR_CURSOR_ROW,
                    );
                }
            }
            if cx.has_key_focus(Some(self.component_id)) {
                let scroll_pos = self.view.get_scroll_pos(cx);
                cx.show_text_ime(rc.x - scroll_pos.x, rc.y - scroll_pos.y);
            } else {
                cx.hide_text_ime();
            }
        }
    }

    fn do_selection_scrolling(&mut self, cx: &mut Cx, text_buffer: &TextBuffer) {
        // do select scrolling
        if let Some(select_scroll) = self._select_scroll.clone() {
            if let Some(grid_select_corner) = self._grid_select_corner {
                // self.cursors.grid_select(offset, text_buffer);
                let pos = self.compute_grid_text_pos_from_abs(cx, select_scroll.abs);
                self.cursors.grid_select(grid_select_corner, pos, text_buffer);
            } else if let Some(offset) =
                TextIns::closest_offset(cx, &self.text_area, select_scroll.abs, TEXT_STYLE_MONO.line_spacing)
            {
                self.cursors.set_last_cursor_head(offset, text_buffer);
            }
            if select_scroll.at_end {
                self._select_scroll = None;
            }
            cx.request_draw();
        }
    }

    fn _do_selection_animations(&mut self, cx: &mut Cx) {
        if !self._anim_folding.state.is_animating() {
            let sel = &mut self._draw_cursors.selections;

            let mut anim_select_any = false;
            for (i, cur) in sel.iter_mut().enumerate() {
                let start_time = if self._select_scroll.is_none() && self._last_pointer_move.is_some() { 1. } else { 0. };
                // silly selection animation start
                if i < self._anim_select.len() && cur.rc.pos.y < self._anim_select[i].ypos {
                    // insert new one at the top
                    self._anim_select.insert(i, AnimSelect { time: start_time, invert: true, ypos: cur.rc.pos.y });
                }
                let (wtime, htime, invert) = if i < self._anim_select.len() {
                    let len = self._anim_select.len() - 1;
                    let anim = &mut self._anim_select[i];
                    anim.ypos = cur.rc.pos.y;
                    if anim.time <= 0.0001 {
                        anim.time = 0.0
                    } else {
                        anim.time *= 0.55;
                        //= 0.1;
                        anim_select_any = true;
                    }
                    if i == len {
                        (anim.time, anim.time, i == 0 && anim.invert)
                    } else {
                        (anim.time, 0., i == 0 && anim.invert)
                    }
                } else {
                    self._anim_select.push(AnimSelect { time: start_time, invert: i == 0, ypos: cur.rc.pos.y });
                    anim_select_any = true;
                    (start_time, start_time, false)
                };
                let wtime = 1.0 - wtime as f32;
                let htime = 1.0 - htime as f32;

                if invert {
                    cur.rc.size.x *= wtime;
                    cur.rc.size.y *= htime;
                } else {
                    cur.rc.pos.x += cur.rc.size.x * (1. - wtime);
                    cur.rc.size.x *= wtime;
                    cur.rc.size.y *= htime;
                }
            }
            self._anim_select.truncate(sel.len());
            if anim_select_any {
                cx.request_draw();
            }
        }
    }

    fn set_indent_line_highlight_id(&mut self, cx: &mut Cx) {
        // compute the line which our last cursor is on so we can set the highlight id
        if !self._indent_line_inst.is_empty() {
            let indent_id = if self.cursors.is_last_cursor_singular() && self._last_cursor_pos.row < self._line_geometry.len() {
                self._line_geometry[self._last_cursor_pos.row].indent_id
            } else {
                0.
            };
            self.indent_lines.set_indent_sel(cx, indent_id);
        }
    }

    // set it once per line otherwise the LineGeom stuff isn't really working out.
    fn set_font_scale(&mut self, _cx: &Cx, font_scale: f32) {
        self.current_font_scale = font_scale;
        if font_scale > self._line_largest_font {
            self._line_largest_font = font_scale;
        }
        self._monospace_size.x = self._monospace_base.x * TEXT_STYLE_MONO.font_size * font_scale;
        self._monospace_size.y = self._monospace_base.y * TEXT_STYLE_MONO.font_size * font_scale;
    }

    pub fn reset_cursors(&mut self) {
        self.cursors = TextCursorSet::default();
    }

    fn scroll_last_cursor_visible(&mut self, cx: &mut Cx, text_buffer: &TextBuffer, height_pad: f32) {
        // so we have to compute (approximately) the rect of our cursor
        if self.cursors.last_cursor >= self.cursors.set.len() {
            panic!("LAST CURSOR INVALID");
        }

        let pos = self.cursors.get_last_cursor_text_pos(text_buffer);

        // alright now lets query the line geometry
        if !self._line_geometry.is_empty() {
            let row = pos.row.min(self._line_geometry.len() - 1);
            if row < self._line_geometry.len() {
                let geom = &self._line_geometry[row];
                let mono_size = Vec2 { x: self._monospace_base.x * geom.font_size, y: self._monospace_base.y * geom.font_size };
                //self.text.get_monospace_size(cx, geom.font_size);
                let rect = Rect {
                    pos: vec2(
                        (pos.col as f32) * mono_size.x, // - self.line_number_width,
                        geom.walk.y - mono_size.y * 1. - 0.5 * height_pad,
                    ),
                    size: vec2(mono_size.x * 4. + self.line_number_width, mono_size.y * 4. + height_pad),
                };

                // scroll this cursor into view
                self.view.scroll_into_view(cx, rect);
            }
        }
    }

    fn scroll_last_cursor_top(&mut self, cx: &mut Cx, text_buffer: &TextBuffer) {
        // so we have to compute (approximately) the rect of our cursor
        if self.cursors.last_cursor >= self.cursors.set.len() {
            panic!("LAST CURSOR INVALID");
        }

        let pos = self.cursors.get_last_cursor_text_pos(text_buffer);

        // alright now lets query the line geometry
        let row = pos.row.min(self._line_geometry.len() - 1);
        if row < self._line_geometry.len() {
            let geom = &self._line_geometry[row];
            let mono_size = Vec2 { x: self._monospace_base.x * geom.font_size, y: self._monospace_base.y * geom.font_size };
            //self.text.get_monospace_size(cx, geom.font_size);
            let rect = Rect {
                pos: vec2(
                    0., // (pos.col as f32) * mono_size.x - self.line_number_width,
                    geom.walk.y - mono_size.y * 1.,
                ),
                size: vec2(mono_size.x * 4. + self.line_number_width, self._final_fill_height + mono_size.y * 1.),
            };

            // scroll this cursor into view
            self.view.scroll_into_view_no_smooth(cx, rect);
        }
    }

    fn compute_grid_text_pos_from_abs(&mut self, cx: &Cx, abs: Vec2) -> TextPos {
        let rel = abs - self.view.area().get_rect_for_first_instance(cx).unwrap_or_default().pos;
        let mut mono_size = Vec2::default();
        for (row, geom) in self._line_geometry.iter().enumerate() {
            //let geom = &self._line_geometry[pos.row];
            mono_size = Vec2 { x: self._monospace_base.x * geom.font_size, y: self._monospace_base.y * geom.font_size };
            if rel.y < geom.walk.y || rel.y >= geom.walk.y && rel.y <= geom.walk.y + mono_size.y {
                // its on the right line
                let col = ((rel.x - self.line_number_width).max(0.) / mono_size.x) as usize;
                // do a dumb calc
                return TextPos { row, col };
            }
        }
        // otherwise the file is too short, lets use the last line
        TextPos { row: self._line_geometry.len() - 1, col: (rel.x.max(0.) / mono_size.x) as usize }
    }

    fn compute_offset_from_ypos(&mut self, cx: &Cx, ypos_abs: f32, text_buffer: &TextBuffer, end: bool) -> usize {
        let rel = ypos_abs - self.view.area().get_rect_for_first_instance(cx).unwrap_or_default().pos.y;
        let mut mono_size;
        // = Vec2::zero();
        let end_col = if end { 1 << 31 } else { 0 };
        for (row, geom) in self._line_geometry.iter().enumerate() {
            //let geom = &self._line_geometry[pos.row];
            mono_size = Vec2 { x: self._monospace_base.x * geom.font_size, y: self._monospace_base.y * geom.font_size };
            if rel < geom.walk.y || rel >= geom.walk.y && rel <= geom.walk.y + mono_size.y {
                // its on the right line
                return text_buffer.text_pos_to_offset(TextPos { row, col: end_col });
            }
        }
        text_buffer.text_pos_to_offset(TextPos { row: self._line_geometry.len() - 1, col: end_col })
    }

    pub fn start_code_folding(&mut self, cx: &mut Cx, text_buffer: &TextBuffer) {
        // start code folding anim
        let speed = 0.98;
        //self._anim_folding.depth = if halfway {1}else {2};
        //self._anim_folding.zoom_scale = if halfway {0.5}else {1.};
        //if halfway{9.0} else{1.0};
        self._anim_folding.state.do_folding(speed, 0.95);
        self._anim_folding.focussed_line = self.compute_focussed_line_for_folding(cx, text_buffer);
        //println!("FOLDING {}",self._anim_folding.focussed_line);
        cx.request_draw();
    }

    pub fn start_code_unfolding(&mut self, cx: &mut Cx, text_buffer: &TextBuffer) {
        let speed = 0.96;
        self._anim_folding.state.do_opening(speed, 0.97);
        self._anim_folding.focussed_line = self.compute_focussed_line_for_folding(cx, text_buffer);
        //println!("UNFOLDING {}",self._anim_folding.focussed_line);
        cx.request_draw();
        // return to normal size
    }

    fn check_select_scroll_dynamics(&mut self, pe: &PointerMoveEvent) -> bool {
        let pow_scale = 0.1;
        let pow_fac = 3.;
        let max_speed = 40.;
        let pad_scroll = 20.;
        let rect = Rect { pos: pe.rect.pos + pad_scroll, size: pe.rect.size - 2. * pad_scroll };
        let delta = Vec2 {
            x: if pe.abs.x < rect.pos.x {
                -((rect.pos.x - pe.abs.x) * pow_scale).powf(pow_fac).min(max_speed)
            } else if pe.abs.x > rect.pos.x + rect.size.x {
                ((pe.abs.x - (rect.pos.x + rect.size.x)) * pow_scale).powf(pow_fac).min(max_speed)
            } else {
                0.
            },
            y: if pe.abs.y < rect.pos.y {
                -((rect.pos.y - pe.abs.y) * pow_scale).powf(pow_fac).min(max_speed)
            } else if pe.abs.y > rect.pos.y + rect.size.y {
                ((pe.abs.y - (rect.pos.y + rect.size.y)) * pow_scale).powf(pow_fac).min(max_speed)
            } else {
                0.
            },
        };
        let last_scroll_none = self._select_scroll.is_none();
        if delta.x != 0. || delta.y != 0. {
            self._select_scroll = Some(SelectScroll { abs: pe.abs, delta, at_end: false });
        } else {
            self._select_scroll = None;
        }
        last_scroll_none
    }

    fn compute_next_unfolded_line_up(&self, text_buffer: &TextBuffer) -> usize {
        let pos = self.cursors.get_last_cursor_text_pos(text_buffer);
        let mut delta = 1;
        if pos.row > 0 && pos.row < self._line_geometry.len() {
            let mut scan = pos.row - 1;
            while scan > 0 {
                if !self._line_geometry[scan].was_folded {
                    delta = pos.row - scan;
                    break;
                }
                scan -= 1;
            }
        };
        delta
    }

    fn compute_next_unfolded_line_down(&self, text_buffer: &TextBuffer) -> usize {
        let pos = self.cursors.get_last_cursor_text_pos(text_buffer);
        let mut delta = 1;
        let mut scan = pos.row + 1;
        while scan < self._line_geometry.len() {
            if !self._line_geometry[scan].was_folded {
                delta = scan - pos.row;
                break;
            }
            scan += 1;
        }
        delta
    }

    fn compute_focussed_line_for_folding(&self, cx: &Cx, text_buffer: &TextBuffer) -> usize {
        let scroll = self.view.get_scroll_pos(cx);
        let rect = self.view.area().get_rect_for_first_instance(cx).unwrap_or_default();

        // first try if our last cursor is in view
        let pos = self.cursors.get_last_cursor_text_pos(text_buffer);
        if pos.row < self._line_geometry.len() {
            let geom = &self._line_geometry[pos.row];
            // check if cursor is visible
            if geom.walk.y - scroll.y > 0. && geom.walk.y - scroll.y < rect.size.y {
                // visible
                //println!("FOUND");
                return pos.row;
            }
        }

        // scan for the centerline otherwise
        let scroll = self.view.get_scroll_pos(cx);
        let center_y = rect.size.y * 0.5 + scroll.y;
        for (line, geom) in self._line_geometry.iter().enumerate() {
            if geom.walk.y > center_y {
                //println!("CENTER");
                return line;
            }
        }

        // if we cant find the centerline, use the view top
        for (line, geom) in self._line_geometry.iter().enumerate() {
            if geom.walk.y > scroll.y {
                //println!("TOP");
                return line;
            }
        }

        // cant find anything
        0
    }
}

#[derive(Clone)]
pub enum AnimFoldingState {
    Open,
    Opening(f32, f32, f32),
    Folded,
    Folding(f32, f32, f32),
}

pub struct AnimFolding {
    pub state: AnimFoldingState,
    pub focussed_line: usize,
    pub did_animate: bool,
}

#[derive(Clone)]
pub struct AnimSelect {
    pub ypos: f32,
    pub invert: bool,
    pub time: f64,
}

#[derive(Clone, Default)]
pub struct LineGeom {
    walk: Vec2,
    was_folded: bool,
    font_size: f32,
    indent_id: f32,
}

#[derive(Clone, Default)]
pub struct SelectScroll {
    // pub margin:Margin,
    pub delta: Vec2,
    pub abs: Vec2,
    pub at_end: bool,
}

#[derive(Clone)]
pub struct ParenItem {
    pair_start: usize,
    geom_open: Option<Rect>,
    geom_close: Option<Rect>,
    marked: bool,
    exp_paren: char,
}

impl AnimFoldingState {
    fn is_animating(&self) -> bool {
        !matches!(self, AnimFoldingState::Open | AnimFoldingState::Folded)
    }

    fn is_folded(&self) -> bool {
        matches!(self, AnimFoldingState::Folded | AnimFoldingState::Folding(_, _, _))
    }

    fn get_font_size(&self, open_size: f32, folded_size: f32) -> f32 {
        match self {
            AnimFoldingState::Open => open_size,
            AnimFoldingState::Folded => folded_size,
            AnimFoldingState::Opening(f, _, _) => f * folded_size + (1. - f) * open_size,
            AnimFoldingState::Folding(f, _, _) => f * open_size + (1. - f) * folded_size,
        }
    }

    fn do_folding(&mut self, speed: f32, speed2: f32) {
        *self = match self {
            AnimFoldingState::Open => AnimFoldingState::Folding(1.0, speed, speed2),
            AnimFoldingState::Folded => AnimFoldingState::Folded,
            AnimFoldingState::Opening(f, _, _) => AnimFoldingState::Folding(1.0 - *f, speed, speed2),
            AnimFoldingState::Folding(f, _, _) => AnimFoldingState::Folding(*f, speed, speed2),
        }
    }

    fn do_opening(&mut self, speed: f32, speed2: f32) {
        *self = match self {
            AnimFoldingState::Open => AnimFoldingState::Open,
            AnimFoldingState::Folded => AnimFoldingState::Opening(1.0, speed, speed2),
            AnimFoldingState::Opening(f, _, _) => AnimFoldingState::Opening(*f, speed, speed2),
            AnimFoldingState::Folding(f, _, _) => AnimFoldingState::Opening(1.0 - *f, speed, speed2),
        }
    }

    fn next_anim_step(&mut self) {
        *self = match self {
            AnimFoldingState::Open => AnimFoldingState::Open,
            AnimFoldingState::Folded => AnimFoldingState::Folded,
            AnimFoldingState::Opening(f, speed, speed2) => {
                let new_f = *f * *speed;
                if new_f < 0.001 {
                    AnimFoldingState::Open
                } else {
                    AnimFoldingState::Opening(new_f, *speed * *speed2, *speed2)
                }
            }
            AnimFoldingState::Folding(f, speed, speed2) => {
                let new_f = *f * *speed;
                if new_f < 0.001 {
                    AnimFoldingState::Folded
                } else {
                    AnimFoldingState::Folding(new_f, *speed * *speed2, *speed2)
                }
            }
        }
    }
}
