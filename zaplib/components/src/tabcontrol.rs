use zaplib::*;

use crate::background::*;
use crate::scrollview::*;
use crate::tab::*;

#[derive(Default)]
pub struct TabControl {
    pub tabs_view: ScrollView,
    pub tabs: Vec<Tab>,
    pub drag_tab_view: View,
    pub drag_tab: Tab,
    pub page_view: View,
    //pub tab_fill_color: ColorId,
    pub tab_fill: Background,

    pub _dragging_tab: Option<(PointerMoveEvent, usize)>,
    pub _tab_id_alloc: usize,
    pub _tab_now_selected: Option<usize>,
    pub _tab_last_selected: Option<usize>,
    pub _focussed: bool,
}

#[derive(Clone, PartialEq)]
pub enum TabControlEvent {
    None,
    TabDragMove { pe: PointerMoveEvent, tab_id: usize },
    TabDragEnd { pe: PointerUpEvent, tab_id: usize },
    TabSelect { tab_id: usize },
    TabClose { tab_id: usize },
}

const COLOR_BG_NORMAL: Vec4 = Vec4::all(0.);

impl TabControl {
    pub fn new() -> Self {
        Self {
            tabs_view: ScrollView::default().with_scroll_h(
                ScrollBarConfig::default().with_bar_size(8.0).with_smoothing(0.15).with_use_vertical_pointer_scroll(true),
            ),

            page_view: View::default(),

            tabs: Default::default(),

            drag_tab: Tab::new().with_draw_depth(10.0),

            drag_tab_view: View::default().with_is_overlay(true),

            //tab_fill_color: Color_bg_normal::id(),
            tab_fill: Background::default(),

            _dragging_tab: None,
            _tab_now_selected: None,
            _tab_last_selected: None,
            _focussed: false,
            _tab_id_alloc: 0,
        }
    }

    pub fn handle_tab_control(&mut self, cx: &mut Cx, event: &mut Event) -> TabControlEvent {
        let mut tab_control_event = TabControlEvent::None;

        self.tabs_view.handle(cx, event);

        for (tab_id, tab) in self.tabs.iter_mut().enumerate() {
            match tab.handle(cx, event) {
                TabEvent::Select => {
                    cx.request_draw();
                    // deselect the other tabs
                    tab_control_event = TabControlEvent::TabSelect { tab_id }
                }
                TabEvent::DragMove(pe) => {
                    self._dragging_tab = Some((pe.clone(), tab_id));
                    // flag our view as dirty, to trigger
                    //cx.request_draw();
                    cx.request_draw();

                    tab_control_event = TabControlEvent::TabDragMove { pe, tab_id };
                }
                TabEvent::DragEnd(pe) => {
                    self._dragging_tab = None;
                    cx.request_draw();

                    tab_control_event = TabControlEvent::TabDragEnd { pe, tab_id };
                }
                TabEvent::Closing => {
                    // this tab is closing. select the visible one
                    if tab.selected() {
                        // only do anything if we are selected
                        let next_sel = if tab_id == self._tab_id_alloc - 1 {
                            // last id
                            if tab_id > 0 {
                                tab_id - 1
                            } else {
                                tab_id
                            }
                        } else {
                            tab_id + 1
                        };
                        if tab_id != next_sel {
                            tab_control_event = TabControlEvent::TabSelect { tab_id: next_sel };
                        }
                    }
                }
                TabEvent::Close => {
                    // Sooooo someone wants to close the tab
                    tab_control_event = TabControlEvent::TabClose { tab_id };
                }
                _ => (),
            }
        }
        match tab_control_event {
            TabControlEvent::TabSelect { tab_id } => {
                self._focussed = true;
                for (id, tab) in self.tabs.iter_mut().enumerate() {
                    if tab_id != id {
                        tab.set_tab_selected(cx, false);
                        tab.set_tab_focus(cx, true);
                    }
                }
            }
            TabControlEvent::TabClose { .. } => {
                // needed to clear animation state
                self.tabs.clear();
            }
            _ => (),
        };
        tab_control_event
    }

    pub fn get_tab_rects(&mut self, cx: &mut Cx) -> Vec<Rect> {
        let mut rects = Vec::new();
        for tab in self.tabs.iter() {
            rects.push(tab.get_tab_rect(cx))
        }
        rects
    }

    pub fn set_tab_control_focus(&mut self, cx: &mut Cx, focus: bool) {
        self._focussed = focus;
        for tab in self.tabs.iter_mut() {
            tab.set_tab_focus(cx, focus);
        }
    }

    pub fn get_tabs_view_rect(&mut self, cx: &Cx) -> Rect {
        self.tabs_view.get_rect(cx)
    }

    pub fn get_content_drop_rect(&mut self, cx: &Cx) -> Rect {
        // we now need to change the y and the new height
        self.page_view.get_rect(cx)
    }

    pub fn begin_tabs(&mut self, cx: &mut Cx) {
        self.tabs_view.begin_view(cx, LayoutSize::new(Width::Fill, Height::Compute));
        cx.begin_row(Width::Fill, Height::Compute);
        self._tab_now_selected = None;
        self._tab_id_alloc = 0;
    }

    pub fn get_draw_tab(&mut self, cx: &mut Cx, label: &str, selected: bool, closeable: bool) -> &mut Tab {
        let new_tab = self.tabs.get(self._tab_id_alloc).is_none();
        if new_tab {
            self.tabs.push(Tab::default());
        }
        let tab = &mut self.tabs[self._tab_id_alloc];
        if selected {
            self._tab_now_selected = Some(self._tab_id_alloc);
        }
        self._tab_id_alloc += 1;
        tab.label = label.to_string();
        tab.is_closeable = closeable;
        if new_tab {
            tab.set_tab_state(cx, selected, self._focussed);
        } else {
            // animate the tabstate
            tab.set_tab_selected(cx, selected);
        }
        tab
    }

    pub fn draw_tab(&mut self, cx: &mut Cx, label: &str, selected: bool, closeable: bool) {
        let tab = self.get_draw_tab(cx, label, selected, closeable);
        tab.draw_tab(cx);
    }

    pub fn end_tabs(&mut self, cx: &mut Cx) {
        self.tab_fill.begin_draw(cx, Width::Fill, Height::Compute, COLOR_BG_NORMAL);
        self.tab_fill.end_draw(cx);

        self.tabs.truncate(self._tab_id_alloc);
        if let Some((pe, id)) = &self._dragging_tab {
            cx.begin_absolute_box();
            self.drag_tab_view.begin_view(cx, LayoutSize::FILL);

            self.drag_tab.abs_origin = Some(Vec2 { x: pe.abs.x - pe.rel_start.x, y: pe.abs.y - pe.rel_start.y });
            let origin_tab = &mut self.tabs[*id];
            self.drag_tab.label = origin_tab.label.clone();
            self.drag_tab.is_closeable = origin_tab.is_closeable;
            self.drag_tab.draw_tab(cx);

            self.drag_tab_view.end_view(cx);
            cx.end_absolute_box();
        }
        cx.end_row();
        self.tabs_view.end_view(cx);

        if self._tab_now_selected != self._tab_last_selected {
            // lets scroll the thing into view
            if let Some(tab_id) = self._tab_now_selected {
                if let Some(tab) = self.tabs.get(tab_id) {
                    let tab_rect = tab.get_tab_rect(cx);
                    self.tabs_view.scroll_into_view_abs(cx, tab_rect);
                }
            }
            self._tab_last_selected = self._tab_now_selected;
        }
    }

    pub fn begin_tab_page(&mut self, cx: &mut Cx) {
        cx.draw_new_line();
        self.page_view.begin_view(cx, LayoutSize::FILL);
    }

    pub fn end_tab_page(&mut self, cx: &mut Cx) {
        self.page_view.end_view(cx);
        // if we are in draggable tab state,
        // draw our draggable tab
    }
}
