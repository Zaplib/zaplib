use crate::buildmanager::*;
use crate::codeicon::*;
use crate::listanims::*;
use crate::makepadstorage::*;
use bigedit_hub::*;
use zaplib::*;
use zaplib_components::*;

pub struct LogList {
    view: ScrollView,
    item_draw: LogItemDraw,
    list: List,
}

#[derive(Clone)]
pub enum LogListEvent {
    SelectLocMessage { loc_message: LocMessage, jump_to_offset: usize },
    SelectMessages { items: String },
    None,
}

const ITEM_HEIGHT: f32 = 20.;

const COLOR_PATH: Vec4 = vec4(0.6, 0.6, 0.6, 1.0);

const ITEM_TEXT_PROPS: TextInsProps = TextInsProps { wrapping: Wrapping::None, color: COLOR_PATH, ..TextInsProps::DEFAULT };

impl LogList {
    pub fn new() -> Self {
        Self {
            item_draw: LogItemDraw::default(),
            list: List::default().with_multi_select(true),
            view: ScrollView::new_standard_vh(),
        }
    }

    fn get_default_anim(counter: usize, marked: bool) -> Anim {
        if marked {
            LIST_ANIMS_ANIM_MARKED
        } else if counter & 1 == 0 {
            LIST_ANIMS_ANIM_EVEN
        } else {
            LIST_ANIMS_ANIM_ODD
        }
    }

    fn get_over_anim(counter: usize, marked: bool) -> Anim {
        if marked {
            LIST_ANIMS_ANIM_MARKED_OVER
        } else if counter & 1 == 0 {
            LIST_ANIMS_ANIM_EVEN_OVER
        } else {
            LIST_ANIMS_ANIM_ODD_OVER
        }
    }

    pub fn handle(
        &mut self,
        cx: &mut Cx,
        event: &mut Event,
        makepad_storage: &mut MakepadStorage,
        bm: &mut BuildManager,
    ) -> LogListEvent {
        self.list.set_list_len(bm.log_items.len());

        if self.list.handle_list_scroll_bars(cx, event, &mut self.view) {
            bm.tail_log_items = false;
        }

        let mut select = ListSelect::None;
        let mut select_at_end = false;
        // global key handle
        match event {
            Event::KeyDown(ke) => match ke.key_code {
                KeyCode::Period => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        select = self.list.get_next_single_selection();
                        self.list.scroll_item_in_view = select.item_index();
                        bm.tail_log_items = false;
                        select_at_end = ke.modifiers.shift;
                    }
                }
                KeyCode::Comma => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        // lets find the
                        select = self.list.get_prev_single_selection();
                        bm.tail_log_items = false;
                        self.list.scroll_item_in_view = select.item_index();
                        select_at_end = ke.modifiers.shift;
                    }
                }
                KeyCode::KeyM => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        select = ListSelect::All;
                    }
                }
                KeyCode::KeyT => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        // lock scroll
                        bm.tail_log_items = true;
                        cx.request_draw();
                    }
                }
                KeyCode::KeyK => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        // clear and tail log
                        bm.tail_log_items = true;
                        bm.log_items.truncate(0);
                        cx.request_draw();
                    }
                }
                _ => (),
            },
            Event::Signal(se) => {
                if let Some(_) = se.signals.get(&bm.signal) {
                    // we have new things
                    cx.request_draw();
                    //println!("SIGNAL!");
                }
            }
            _ => (),
        }

        let item_draw = &mut self.item_draw;
        let le = self.list.handle_list_logic(cx, event, select, false, |cx, item_event, item, item_index| match item_event {
            ListLogicEvent::Animate => {
                item_draw.animate(cx, item.area(), &mut item.animator);
            }
            ListLogicEvent::Select => {
                item.animator.play_anim(cx, Self::get_over_anim(item_index, true));
            }
            ListLogicEvent::Deselect => {
                item.animator.play_anim(cx, Self::get_default_anim(item_index, false));
            }
            ListLogicEvent::Cleanup => {
                item.animator.play_anim(cx, Self::get_default_anim(item_index, item.is_selected));
            }
            ListLogicEvent::Over => {
                item.animator.play_anim(cx, Self::get_over_anim(item_index, item.is_selected));
            }
            ListLogicEvent::Out => {
                item.animator.play_anim(cx, Self::get_default_anim(item_index, item.is_selected));
            }
        });

        match le {
            ListEvent::SelectSingle(select_index) => {
                cx.request_draw();
                let log_item = &bm.log_items[select_index];
                if let Some(loc_message) = log_item.get_loc_message() {
                    if loc_message.path.is_empty() {
                        return LogListEvent::SelectLocMessage { loc_message: loc_message.clone(), jump_to_offset: 0 };
                    }

                    let text_buffer = &makepad_storage
                        .text_buffer_from_path(cx, &makepad_storage.remap_sync_path(&loc_message.path))
                        .text_buffer;
                    // check if we have a range:
                    let offset = if let Some((head, tail)) = loc_message.range {
                        if select_at_end {
                            head
                        } else {
                            tail
                        }
                    } else {
                        text_buffer
                            .text_pos_to_offset(TextPos { row: loc_message.line.max(1) - 1, col: loc_message.column.max(1) - 1 })
                    };

                    LogListEvent::SelectLocMessage { loc_message: loc_message.clone(), jump_to_offset: offset }
                } else {
                    LogListEvent::SelectMessages { items: log_item.get_body().clone() }
                }
            }
            ListEvent::SelectMultiple => {
                cx.request_draw();
                let mut items = String::new();
                for select in &self.list.selection {
                    if let Some(loc_message) = bm.log_items[*select].get_loc_message() {
                        if let Some(rendered) = &loc_message.rendered {
                            items.push_str(rendered);
                            if items.len() > 1000000 {
                                // safety break
                                break;
                            }
                        }
                    } else {
                        items.push_str(bm.log_items[*select].get_body());
                        if items.len() > 1000000 {
                            // safety break
                            break;
                        }
                    }
                }

                LogListEvent::SelectMessages { items }
            }
            ListEvent::SelectDouble(_) | ListEvent::None => LogListEvent::None,
        }
    }

    pub fn draw(&mut self, cx: &mut Cx, bm: &BuildManager) {
        self.list.set_list_len(bm.log_items.len());

        let row_height = ITEM_HEIGHT;

        self.list.begin_list(cx, &mut self.view, bm.tail_log_items, row_height);

        let mut counter = 0;
        for i in self.list.start_item..self.list.end_item {
            self.item_draw.draw_log_item(cx, i, &mut self.list.list_items[i], &bm.log_items[i], Self::get_default_anim(0, false));
            counter += 1;
        }

        self.list.walk_box_to_end(cx, row_height);

        self.item_draw.draw_status_line(cx, counter, bm);
        counter += 1;

        // draw filler nodes
        for _ in (self.list.end_item + 1)..self.list.end_fill {
            self.item_draw.draw_filler(cx, counter);
            counter += 1;
        }

        ScrollShadow::draw_shadow_left(cx, 0.01);
        ScrollShadow::draw_shadow_top(cx, 0.01);

        self.list.end_list(cx, &mut self.view);
    }
}

#[derive(Default)]
struct LogItemDraw {
    item_bg: Background,
}

impl LogItemDraw {
    fn draw_icon(&mut self, cx: &mut Cx, icon_type: CodeIconType) {
        cx.begin_row(Width::Compute, Height::Fix(ITEM_HEIGHT));
        cx.begin_center_y_align();
        CodeIconIns::draw(cx, icon_type);
        cx.end_center_y_align();
        cx.end_row();
    }

    fn draw_log_path(&mut self, cx: &mut Cx, path: &str, row: usize) {
        cx.begin_row(Width::Compute, Height::Fix(ITEM_HEIGHT));
        cx.begin_center_y_align();
        TextIns::draw_walk(cx, &format!("{}:{} - ", path, row), &ITEM_TEXT_PROPS);
        cx.end_center_y_align();
        cx.end_row();
    }

    fn draw_log_body(&mut self, cx: &mut Cx, body: &str) {
        cx.begin_row(Width::Compute, Height::Fix(ITEM_HEIGHT));
        cx.begin_center_y_align();
        let text = if body.len() > 500 { &body[0..500] } else { body };
        let item_text_props = TextInsProps { color: vec4(0.73, 0.73, 0.73, 1.0), ..ITEM_TEXT_PROPS };
        TextIns::draw_walk(cx, text, &item_text_props);
        cx.end_center_y_align();
        cx.end_row();
    }

    fn animate(&mut self, cx: &mut Cx, area: Area, animator: &mut Animator) {
        self.item_bg.set_area(area);
        self.item_bg.set_color(cx, animator.get_vec4(0));
    }

    fn draw_log_item(&mut self, cx: &mut Cx, _index: usize, list_item: &mut ListItem, log_item: &HubLogItem, anim_default: Anim) {
        self.item_bg.set_area(list_item.area());
        self.item_bg.begin_draw(cx, Width::Fill, Height::Fix(ITEM_HEIGHT), Vec4::default());
        list_item.set_area(self.item_bg.area());

        list_item.animator.draw(cx, anim_default);
        self.animate(cx, list_item.area(), &mut list_item.animator);

        match log_item {
            HubLogItem::LocPanic(loc_msg) => {
                self.draw_icon(cx, CodeIconType::Panic);
                self.draw_log_path(cx, &loc_msg.path, loc_msg.line);
                self.draw_log_body(cx, &loc_msg.body);
            }
            HubLogItem::LocError(loc_msg) => {
                self.draw_icon(cx, CodeIconType::Error);
                self.draw_log_path(cx, &loc_msg.path, loc_msg.line);
                self.draw_log_body(cx, &loc_msg.body);
            }
            HubLogItem::LocWarning(loc_msg) => {
                self.draw_icon(cx, CodeIconType::Warning);
                self.draw_log_path(cx, &loc_msg.path, loc_msg.line);
                self.draw_log_body(cx, &loc_msg.body);
            }
            HubLogItem::LocMessage(loc_msg) => {
                self.draw_log_path(cx, &loc_msg.path, loc_msg.line);
                self.draw_log_body(cx, &loc_msg.body);
            }
            HubLogItem::Error(msg) => {
                self.draw_icon(cx, CodeIconType::Error);
                self.draw_log_body(cx, msg);
            }
            HubLogItem::Warning(msg) => {
                self.draw_icon(cx, CodeIconType::Warning);
                self.draw_log_body(cx, msg);
            }
            HubLogItem::Message(msg) => {
                self.draw_log_body(cx, msg);
            }
        }
        self.item_bg.end_draw(cx);
        list_item.set_area(self.item_bg.area());
    }

    fn draw_status_line(&mut self, cx: &mut Cx, counter: usize, bm: &BuildManager) {
        self.item_bg.begin_draw(
            cx,
            Width::Fill,
            Height::Fix(ITEM_HEIGHT),
            if counter & 1 == 0 { LIST_ANIMS_COLOR_BG_EVEN } else { LIST_ANIMS_COLOR_BG_ODD },
        );
        cx.begin_center_y_align();

        if !bm.is_any_cargo_running() {
            self.draw_icon(cx, CodeIconType::Ok);
            cx.begin_row(Width::Compute, Height::Fix(ITEM_HEIGHT));
            cx.begin_center_y_align();
            if bm.is_any_artifact_running() {
                TextIns::draw_walk(cx, "Running ", &ITEM_TEXT_PROPS);
                for ab in &bm.active_builds {
                    if ab.run_uid.is_some() {
                        let bt = &ab.build_target;
                        TextIns::draw_walk(
                            cx,
                            &format!("{}/{}/{}:{} ", bt.builder, bt.workspace, bt.package, bt.config),
                            &ITEM_TEXT_PROPS,
                        );
                    }
                }
            } else {
                TextIns::draw_walk(cx, "Done ", &ITEM_TEXT_PROPS);
                for ab in &bm.active_builds {
                    let bt = &ab.build_target;
                    TextIns::draw_walk(
                        cx,
                        &format!("{}/{}/{}:{} ", bt.builder, bt.workspace, bt.package, bt.config),
                        &ITEM_TEXT_PROPS,
                    );
                }
            }
            cx.end_center_y_align();
            cx.end_row();
        } else {
            CodeIconIns::draw(cx, CodeIconType::Wait);

            cx.begin_row(Width::Compute, Height::Fix(ITEM_HEIGHT));
            cx.begin_center_y_align();
            TextIns::draw_walk(cx, &format!("Building ({}) ", bm.artifacts.len()), &ITEM_TEXT_PROPS);
            for ab in &bm.active_builds {
                if ab.build_uid.is_some() {
                    let bt = &ab.build_target;
                    TextIns::draw_walk(
                        cx,
                        &format!("{}/{}/{}:{} ", bt.builder, bt.workspace, bt.package, bt.config),
                        &ITEM_TEXT_PROPS,
                    );
                }
            }
            if bm.exec_when_done {
                TextIns::draw_walk(cx, " - starting when done", &ITEM_TEXT_PROPS);
            }
            cx.end_center_y_align();
            cx.end_row();
        }
        cx.end_center_y_align();
        self.item_bg.end_draw(cx);
    }

    fn draw_filler(&mut self, cx: &mut Cx, counter: usize) {
        self.item_bg.begin_draw(
            cx,
            Width::Fill,
            // Draw filler node of fixed height, until hitting the visibile boundaries, to prevent unneccesary scrolling
            Height::FillUntil(ITEM_HEIGHT),
            if counter & 1 == 0 { LIST_ANIMS_COLOR_BG_EVEN } else { LIST_ANIMS_COLOR_BG_ODD },
        );
        self.item_bg.end_draw(cx);
    }
}
