use crate::makepadstorage::*;
use crate::searchindex::*;
use zaplib::*;
use zaplib_components::*;

pub struct SearchResults {
    view: ScrollView,
    result_draw: SearchResultDraw,
    list: List,
    search_input: TextInput,
    do_select_first: bool,
    first_tbid: MakepadTextBufferId,
    results: Vec<SearchResult>,
}

struct SearchResultDraw {
    text_editor: TextEditor,
    item_bg: Background,
}

#[derive(Clone)]
pub enum SearchResultEvent {
    DisplayFile { text_buffer_id: MakepadTextBufferId, cursor: (usize, usize) },
    OpenFile { text_buffer_id: MakepadTextBufferId, cursor: (usize, usize) },
    None,
}

const LAYOUT_ITEM_PADDING: Padding = Padding { l: 5., t: 3., b: 2., r: 0. };
const ITEM_CLOSED_HEIGHT: f32 = 37.;

const ANIM_UNMARKED: Anim = Anim {
    duration: 0.01,
    chain: true,
    tracks: &[Track::Vec4 { key_frames: &[(1.0, vec4(0.12, 0.12, 0.12, 1.0))], ease: Ease::DEFAULT }],
};
const ANIM_MARKED: Anim = Anim {
    duration: 0.01,
    chain: true,
    tracks: &[Track::Vec4 { key_frames: &[(1.0, vec4(0.07, 0.27, 0.43, 1.0))], ease: Ease::DEFAULT }],
};
const ANIM_UNMARKED_OVER: Anim = Anim {
    duration: 0.02,
    chain: true,
    tracks: &[Track::Vec4 { key_frames: &[(0.0, vec4(0.2, 0.2, 0.2, 1.0))], ease: Ease::DEFAULT }],
};
const ANIM_MARKED_OVER: Anim = Anim {
    duration: 0.02,
    chain: true,
    tracks: &[Track::Vec4 { key_frames: &[(0.0, vec4(0.07, 0.27, 0.43, 1.0))], ease: Ease::DEFAULT }],
};

impl SearchResults {
    pub fn new() -> Self {
        Self {
            first_tbid: MakepadTextBufferId(0),
            search_input: TextInput::new(TextInputOptions {
                multiline: false,
                read_only: false,
                empty_message: "search".to_string(),
            }),
            result_draw: SearchResultDraw::new(),
            list: List::default(),
            do_select_first: false,
            view: ScrollView::new_standard_vh(),
            results: Vec::new(),
        }
    }

    pub fn set_search_input_value(&mut self, cx: &mut Cx, value: &str, first_tbid: MakepadTextBufferId, focus: bool) {
        self.search_input.set_value(cx, value);
        self.first_tbid = first_tbid;
        if focus {
            self.search_input.text_editor.set_key_focus(cx);
        }
        self.search_input.select_all(cx);
    }

    pub fn do_search(
        &mut self,
        cx: &mut Cx,
        search_index: &mut SearchIndex,
        makepad_storage: &mut MakepadStorage,
    ) -> Option<(MakepadTextBufferId, (usize, usize))> {
        let s = self.search_input.get_value();
        if !s.is_empty() {
            // lets search
            self.results = search_index.search(&s, self.first_tbid, cx, makepad_storage);
            self.do_select_first = true;
        } else {
            search_index.clear_markers(cx, makepad_storage);
            self.results.truncate(0);
        }
        self.list.set_list_len(0);
        cx.request_draw();
        if !self.results.is_empty() {
            let result = &self.results[0];
            let text_buffer = &mut makepad_storage.text_buffers[result.text_buffer_id.as_index()].text_buffer;
            let tok = &text_buffer.token_chunks[result.token as usize];
            Some((result.text_buffer_id, (tok.offset + tok.len, tok.offset)))
        } else {
            None
        }
    }

    pub fn handle_search_input(
        &mut self,
        cx: &mut Cx,
        event: &mut Event,
        search_index: &mut SearchIndex,
        makepad_storage: &mut MakepadStorage,
    ) -> bool {
        // if we have a text change, do a search.
        match self.search_input.handle(cx, event) {
            TextEditorEvent::KeyFocus => return true,
            TextEditorEvent::Change => {
                self.do_search(cx, search_index, makepad_storage);
                return true;
            }
            TextEditorEvent::Escape | TextEditorEvent::Search(_) => {
                cx.revert_key_focus();
            }
            _ => (),
        }
        false
    }

    pub fn handle_search_results(
        &mut self,
        cx: &mut Cx,
        event: &mut Event,
        _search_index: &mut SearchIndex,
        makepad_storage: &mut MakepadStorage,
    ) -> SearchResultEvent {
        self.list.set_list_len(self.results.len());

        if self.list.handle_list_scroll_bars(cx, event, &mut self.view) {}

        let mut select = ListSelect::None;
        let mut dblclick = false;
        // global key handle
        if let Event::KeyDown(ke) = event {
            if self.search_input.text_editor.has_key_focus(cx) {
                match ke.key_code {
                    KeyCode::ArrowDown => {
                        select = self.list.get_next_single_selection();
                        self.list.scroll_item_in_view = select.item_index();
                    }
                    KeyCode::ArrowUp => {
                        // lets find the
                        select = self.list.get_prev_single_selection();
                        self.list.scroll_item_in_view = select.item_index();
                    }
                    KeyCode::Return => {
                        if !self.list.selection.is_empty() {
                            select = ListSelect::Single(self.list.selection[0]);
                            dblclick = true;
                        }
                    }
                    _ => (),
                }
            }
        }

        if self.do_select_first {
            self.do_select_first = false;
            select = ListSelect::Single(0);
        }
        let result_draw = &mut self.result_draw;

        let le = self.list.handle_list_logic(cx, event, select, dblclick, |cx, item_event, item, _item_index| match item_event {
            ListLogicEvent::Animate => {
                result_draw.animate(cx, item.area(), &mut item.animator);
            }
            ListLogicEvent::Select => {
                item.animator.play_anim(cx, SearchResultDraw::get_over_anim(true));
            }
            ListLogicEvent::Deselect => {
                item.animator.play_anim(cx, SearchResultDraw::get_default_anim(false));
            }
            ListLogicEvent::Cleanup => {
                item.animator.play_anim(cx, SearchResultDraw::get_default_anim(item.is_selected));
            }
            ListLogicEvent::Over => {
                item.animator.play_anim(cx, SearchResultDraw::get_over_anim(item.is_selected));
            }
            ListLogicEvent::Out => {
                item.animator.play_anim(cx, SearchResultDraw::get_default_anim(item.is_selected));
            }
        });

        match le {
            ListEvent::SelectSingle(select_index) => {
                cx.request_draw();
                let result = &self.results[select_index];
                let text_buffer = &mut makepad_storage.text_buffers[result.text_buffer_id.as_index()].text_buffer;
                if let Event::PointerDown(_) = event {
                    self.search_input.text_editor.set_key_focus(cx);
                }
                let tok = &text_buffer.token_chunks[result.token as usize];
                return SearchResultEvent::DisplayFile {
                    text_buffer_id: result.text_buffer_id,
                    cursor: (tok.offset + tok.len, tok.offset),
                };
            }
            ListEvent::SelectDouble(select_index) => {
                // we need to get a filepath
                let result = &self.results[select_index];
                let text_buffer = &mut makepad_storage.text_buffers[result.text_buffer_id.as_index()].text_buffer;
                let tok = &text_buffer.token_chunks[result.token as usize];
                return SearchResultEvent::OpenFile {
                    text_buffer_id: result.text_buffer_id,
                    cursor: (tok.offset + tok.len, tok.offset),
                };
            }
            ListEvent::SelectMultiple => {}
            ListEvent::None => {}
        }
        SearchResultEvent::None
    }

    pub fn draw_search_result_tab(&mut self, cx: &mut Cx, _search_index: &SearchIndex) {
        cx.move_draw_pos(0., 2.);
        self.search_input.draw(cx);
    }

    pub fn draw_search_results(&mut self, cx: &mut Cx, makepad_storage: &MakepadStorage) {
        self.list.set_list_len(self.results.len());

        let row_height = ITEM_CLOSED_HEIGHT;

        self.list.begin_list(cx, &mut self.view, false, row_height);

        self.result_draw.text_editor.apply_style();
        self.result_draw.text_editor.begin_draw_objects();

        let mut counter = 0;
        for i in self.list.start_item..self.list.end_item {
            // lets get the path
            let result = &self.results[i];
            let tb = &makepad_storage.text_buffers[result.text_buffer_id.as_index()];
            //println!("{} {}");
            self.result_draw.draw_result(cx, i, &mut self.list.list_items[i], &tb.full_path, &tb.text_buffer, result.token);
            counter += 1;
        }

        self.list.walk_box_to_end(cx, row_height);

        // draw filler nodes
        for _ in (self.list.end_item + 1)..self.list.end_fill {
            self.result_draw.draw_filler(cx, counter);
            counter += 1;
        }

        ScrollShadow::draw_shadow_left(cx, 0.01);
        ScrollShadow::draw_shadow_top(cx, 0.01);

        self.result_draw.text_editor.end_draw_objects(cx);

        self.list.end_list(cx, &mut self.view);
    }
}

impl SearchResultDraw {
    fn new() -> Self {
        Self {
            text_editor: TextEditor {
                mark_unmatched_parens: false,
                draw_line_numbers: false,
                line_number_width: 10.,
                top_padding: 0.,
                ..TextEditor::default()
            },
            item_bg: Background::default(),
        }
    }

    fn animate(&mut self, cx: &mut Cx, area: Area, animator: &mut Animator) {
        self.item_bg.set_area(area);
        self.item_bg.set_color(cx, animator.get_vec4(0));
    }

    fn get_default_anim(marked: bool) -> Anim {
        if marked {
            ANIM_MARKED
        } else {
            ANIM_UNMARKED
        }
    }

    fn get_over_anim(marked: bool) -> Anim {
        if marked {
            ANIM_MARKED_OVER
        } else {
            ANIM_UNMARKED_OVER
        }
    }

    fn draw_result(
        &mut self,
        cx: &mut Cx,
        _index: usize,
        list_item: &mut ListItem,
        path: &str,
        text_buffer: &TextBuffer,
        token: u32,
    ) {
        list_item.animator.draw(cx, ANIM_UNMARKED);

        let selected = list_item.is_selected;
        self.item_bg.set_area(list_item.area());

        self.item_bg.begin_draw(
            cx,
            Width::Fill,
            if selected { Height::Fix(85.) } else { Height::Fix(ITEM_CLOSED_HEIGHT) },
            Vec4::default(),
        );
        cx.begin_padding_box(LAYOUT_ITEM_PADDING);
        cx.begin_center_y_align();

        list_item.set_area(self.item_bg.area());
        self.animate(cx, list_item.area(), &mut list_item.animator);

        let window_up = if selected { 2 } else { 1 };
        let window_down = if selected { 3 } else { 1 };
        let (first_tok, delta) = text_buffer.scan_token_chunks_prev_line(token as usize, window_up);
        let last_tok = text_buffer.scan_token_chunks_next_line(token as usize, window_down);

        let tok = &text_buffer.token_chunks[token as usize];
        let pos = text_buffer.offset_to_text_pos(tok.offset);

        let split = path.split('/').collect::<Vec<&str>>();
        TextIns::draw_walk(
            cx,
            &format!("{}:{} - {}", split.last().unwrap(), pos.row, split[0..split.len() - 1].join("/")),
            &TextInsProps { wrapping: Wrapping::Word, color: vec4(0.6, 0.6, 0.6, 1.0), ..TextInsProps::DEFAULT },
        );
        cx.draw_new_line();
        cx.move_draw_pos(0., 5.);

        self.text_editor.search_markers_bypass.truncate(0);
        self.text_editor.search_markers_bypass.push(TextCursor { tail: tok.offset, head: tok.offset + tok.len, max: 0 });

        self.text_editor.line_number_offset = (pos.row as isize + delta) as usize;
        self.text_editor.init_draw_state(cx, text_buffer);

        let mut first_ws = !selected;
        for index in first_tok..last_tok {
            let token_chunk = &text_buffer.token_chunks[index];
            if first_ws && token_chunk.token_type == TokenType::Whitespace {
                continue;
            } else {
                first_ws = false;
            }
            self.text_editor.draw_chunk(cx, index, &text_buffer.flat_text, token_chunk, &text_buffer.markers);
        }

        self.text_editor.draw_search_markers(cx);
        // ok now we have to draw a code bubble
        // its the 3 lines it consists of so.. we have to scan 'back from token to find the previous start
        // and scan to end

        //println!("{}", result.text_buffer_id.0);
        cx.end_center_y_align();
        cx.end_padding_box();
        self.item_bg.end_draw(cx);
    }

    fn draw_filler(&mut self, cx: &mut Cx, counter: usize) {
        self.item_bg.begin_draw(
            cx,
            Width::Fill,
            // Draw filler node of fixed height, until hitting the visibile boundaries, to prevent unnecessary scrolling
            Height::FillUntil(ITEM_CLOSED_HEIGHT),
            if counter & 1 == 0 { vec4(0.16, 0.16, 0.16, 1.0) } else { vec4(0.15, 0.15, 0.15, 1.0) },
        );
        self.item_bg.end_draw(cx);
    }
}
