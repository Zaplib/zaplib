use std::mem;

use crate::axis::*;
use crate::background::*;
use crate::splitter::*;
use crate::tabcontrol::*;
use std::collections::HashMap;
use zaplib::*;

pub struct Dock<TItem>
where
    TItem: Clone,
{
    // pub dock_items: Option<DockItem<TItem>>,
    splitters: Elements<usize, Splitter>,
    tab_controls: Elements<usize, TabControl>,

    pub drop_size: Vec2,
    pub drop_quad: Background,
    pub drop_quad_view: View,
    //pub drop_quad_color: ColorId,
    pub _drag_move: Option<PointerMoveEvent>,
    pub _drag_end: Option<DockDragEnd<TItem>>,
    pub _close_tab: Option<DockTabIdent>,
    pub _tab_select: Option<(usize, usize)>,
    //pub _tweening_quad: Option<(usize, Rect, f32)>
}

#[derive(Clone, Debug)]
pub struct DockTabIdent {
    tab_control_id: usize,
    tab_id: usize,
}

#[derive(Clone)]
pub enum DockDragEnd<TItem>
where
    TItem: Clone,
{
    OldTab { pe: PointerUpEvent, ident: DockTabIdent },
    NewItems { pe: PointerUpEvent, items: Vec<DockTab<TItem>> },
}

#[derive(Clone)]
pub struct DockTab<TItem>
where
    TItem: Clone,
{
    pub closeable: bool,
    pub title: String,
    pub item: TItem,
}

#[derive(Clone)]
pub enum DockItem<TItem>
where
    TItem: Clone,
{
    TabControl { current: usize, previous: usize, tabs: Vec<DockTab<TItem>> },
    Splitter { pos: f32, align: SplitterAlign, axis: Axis, first: Box<DockItem<TItem>>, last: Box<DockItem<TItem>> },
}

struct DockWalkStack<'a, TItem>
where
    TItem: Clone,
{
    counter: usize,
    uid: usize,
    item: &'a mut DockItem<TItem>,
}

pub enum DockEvent<TItem> {
    None,
    DockTabClosed(TItem),
    DockTabCloned { tab_control_id: usize, tab_id: usize },
    DockChanged,
}

pub struct DockWalker<'a, TItem>
where
    TItem: Clone,
{
    walk_uid: usize,
    stack: Vec<DockWalkStack<'a, TItem>>,
    // forwards for Dock
    splitters: &'a mut Elements<usize, Splitter>,
    tab_controls: &'a mut Elements<usize, TabControl>,
    _drag_move: &'a mut Option<PointerMoveEvent>,
    _drag_end: &'a mut Option<DockDragEnd<TItem>>,
    _close_tab: &'a mut Option<DockTabIdent>,
    _tab_select: &'a mut Option<(usize, usize)>,
}

impl<'a, TItem> DockWalker<'a, TItem>
where
    TItem: Clone,
{
    pub fn walk_dock_item(&mut self) -> Option<(usize, &mut DockItem<TItem>)> {
        // lets get the current item on the stack
        let push_or_pop = if let Some(stack_top) = self.stack.last_mut() {
            // return item 'count'
            match stack_top.item {
                /*
                DockItem::Single(..) => {
                    if stack_top.counter == 0 {
                        stack_top.counter += 1;
                        return Some((self.walk_uid, unsafe {mem::transmute(&mut *stack_top.item)}));
                    }
                    else {
                        None
                    }
                },*/
                DockItem::TabControl { .. } => {
                    if stack_top.counter == 0 {
                        let uid = self.walk_uid;
                        self.walk_uid += 1;
                        stack_top.counter += 1;
                        return Some((uid, unsafe { mem::transmute(&mut *stack_top.item) }));
                    } else {
                        None
                    }
                }
                DockItem::Splitter { first, last, .. } => {
                    if stack_top.counter == 0 {
                        let uid = self.walk_uid;
                        self.walk_uid += 1;
                        stack_top.counter += 1;
                        return Some((uid, unsafe { mem::transmute(&mut *stack_top.item) }));
                    } else if stack_top.counter == 1 {
                        stack_top.counter += 1;
                        Some(DockWalkStack { counter: 0, uid: 0, item: unsafe { mem::transmute(first.as_mut()) } })
                    } else if stack_top.counter == 2 {
                        stack_top.counter += 1;
                        Some(DockWalkStack { counter: 0, uid: 0, item: unsafe { mem::transmute(last.as_mut()) } })
                    } else {
                        None
                    }
                }
            }
        } else {
            return None;
        };
        if let Some(item) = push_or_pop {
            self.stack.push(item);
            return self.walk_dock_item();
        } else if !self.stack.is_empty() {
            self.stack.pop();
            return self.walk_dock_item();
        }
        None
    }

    pub fn walk_handle_dock(&mut self, cx: &mut Cx, event: &mut Event) -> Option<(&mut TItem, DockTabIdent)> {
        // lets get the current item on the stack
        let push_or_pop = if let Some(stack_top) = self.stack.last_mut() {
            // return item 'count'
            match stack_top.item {
                /*
                DockItem::Single(item) => {
                    if stack_top.counter == 0 {
                        stack_top.counter += 1;
                        return Some(unsafe {mem::transmute(item)});
                    }
                    else {
                        None
                    }
                },*/
                DockItem::TabControl { current, previous, tabs } => {
                    if stack_top.counter == 0 {
                        let tab_control_id = self.walk_uid;
                        stack_top.counter += 1;
                        stack_top.uid = self.walk_uid;
                        self.walk_uid += 1;

                        if *current < tabs.len() {
                            return Some((
                                unsafe { mem::transmute(&mut tabs[*current].item) },
                                DockTabIdent { tab_control_id, tab_id: *current },
                            ));
                        }
                        None
                    } else {
                        let tab_control_option = self.tab_controls.get_mut(stack_top.uid);
                        let mut defocus = false;
                        if let Some(tab_control) = tab_control_option {
                            match tab_control.handle_tab_control(cx, event) {
                                TabControlEvent::TabSelect { tab_id } => {
                                    if *current != tab_id {
                                        *previous = *current;
                                        *current = tab_id;
                                        // someday ill fix this. Luckily entire UI redraws are millisecond span
                                        cx.request_draw();
                                        *self._tab_select = Some((stack_top.uid, tab_id));
                                        defocus = true;
                                    }
                                }
                                TabControlEvent::TabDragMove { pe, .. } => {
                                    *self._drag_move = Some(pe);
                                    *self._drag_end = None;
                                    cx.request_draw();
                                }
                                TabControlEvent::TabDragEnd { pe, tab_id } => {
                                    *self._drag_move = None;
                                    *self._drag_end = Some(DockDragEnd::OldTab {
                                        pe,

                                        ident: DockTabIdent { tab_control_id: stack_top.uid, tab_id },
                                    });
                                    cx.request_draw();
                                }
                                TabControlEvent::TabClose { tab_id } => {
                                    *self._close_tab = Some(DockTabIdent { tab_control_id: stack_top.uid, tab_id });
                                    // if tab_id < current, subtract current if >0
                                    if tab_id < *current && *current > 0 {
                                        *current -= 1;
                                    }
                                    cx.request_draw();
                                }
                                _ => (),
                            }
                        }

                        if defocus {
                            // defocus all other tabcontrols
                            for (id, tab_control) in self.tab_controls.enumerate() {
                                if *id != stack_top.uid {
                                    tab_control.set_tab_control_focus(cx, false);
                                }
                            }
                        }

                        None
                    }
                }
                DockItem::Splitter { first, last, pos, align, .. } => {
                    if stack_top.counter == 0 {
                        stack_top.counter += 1;
                        stack_top.uid = self.walk_uid;
                        self.walk_uid += 1;
                        let split = self.splitters.get_mut(stack_top.uid);
                        if let Some(split) = split {
                            match split.handle(cx, event) {
                                SplitterEvent::Moving { new_pos } => {
                                    *pos = new_pos;
                                    cx.request_draw();
                                }
                                SplitterEvent::MovingEnd { new_align, new_pos } => {
                                    *align = new_align;
                                    *pos = new_pos;
                                    cx.request_draw();
                                }
                                _ => (),
                            };
                        }
                        // update state in our splitter level
                        Some(DockWalkStack { counter: 0, uid: 0, item: unsafe { mem::transmute(first.as_mut()) } })
                    } else if stack_top.counter == 1 {
                        stack_top.counter += 1;
                        Some(DockWalkStack { counter: 0, uid: 0, item: unsafe { mem::transmute(last.as_mut()) } })
                    } else {
                        None
                    }
                }
            }
        } else {
            return None;
        };
        if let Some(item) = push_or_pop {
            self.stack.push(item);
            return self.walk_handle_dock(cx, event);
        } else if !self.stack.is_empty() {
            self.stack.pop();
            return self.walk_handle_dock(cx, event);
        }
        None
    }

    pub fn walk_draw_dock<F>(&mut self, cx: &mut Cx, mut tab_handler: F) -> Option<&'a mut TItem>
    where
        F: FnMut(&mut Cx, &mut TabControl, &DockTab<TItem>, bool),
    {
        // lets get the current item on the stack
        let push_or_pop = if let Some(stack_top) = self.stack.last_mut() {
            // return item 'count'
            match stack_top.item {
                /*
                DockItem::Single(item) => {
                    if stack_top.counter == 0 {
                        stack_top.counter += 1;
                        return Some(unsafe {mem::transmute(item)});
                    }
                    else {
                        None
                    }
                },*/
                DockItem::TabControl { current, previous: _, tabs } => {
                    if stack_top.counter == 0 {
                        stack_top.counter += 1;
                        stack_top.uid = self.walk_uid;
                        self.walk_uid += 1;
                        let tab_control = self.tab_controls.get_draw(stack_top.uid, TabControl::new);

                        tab_control.begin_tabs(cx);
                        for (id, tab) in tabs.iter().enumerate() {
                            tab_handler(cx, tab_control, tab, *current == id)
                        }
                        tab_control.end_tabs(cx);

                        tab_control.begin_tab_page(cx);
                        if *current < tabs.len() {
                            return Some(unsafe { mem::transmute(&mut tabs[*current].item) });
                        }
                        tab_control.end_tab_page(cx);
                        None
                    } else {
                        let tab_control = self.tab_controls.get_draw(stack_top.uid, TabControl::new);
                        tab_control.end_tab_page(cx);
                        None
                    }
                }
                DockItem::Splitter { align, pos, axis, first, last } => {
                    if stack_top.counter == 0 {
                        stack_top.counter += 1;
                        stack_top.uid = self.walk_uid;
                        self.walk_uid += 1;
                        // begin a split
                        let split = self.splitters.get_draw(stack_top.uid, Splitter::default);
                        split.set_splitter_state(align.clone(), *pos, *axis);
                        split.begin_draw(cx);
                        Some(DockWalkStack { counter: 0, uid: 0, item: unsafe { mem::transmute(first.as_mut()) } })
                    } else if stack_top.counter == 1 {
                        stack_top.counter += 1;

                        let split = self.splitters.get_draw(stack_top.uid, Splitter::default);
                        split.mid_draw(cx);

                        Some(DockWalkStack { counter: 0, uid: 0, item: unsafe { mem::transmute(last.as_mut()) } })
                    } else {
                        let split = self.splitters.get_draw(stack_top.uid, Splitter::default);
                        split.end_draw(cx);
                        None
                    }
                }
            }
        } else {
            return None;
        };
        if let Some(item) = push_or_pop {
            self.stack.push(item);
            return self.walk_draw_dock(cx, tab_handler);
        } else if !self.stack.is_empty() {
            self.stack.pop();
            return self.walk_draw_dock(cx, tab_handler);
        }
        None
    }
}

enum DockDropKind {
    Tab(usize),
    TabsView,
    Left,
    Top,
    Right,
    Bottom,
    Center,
}

impl<TItem> Default for Dock<TItem>
where
    TItem: Clone,
{
    fn default() -> Dock<TItem> {
        Dock {
            // dock_items:None,
            drop_size: Vec2 { x: 100., y: 70. },
            //drop_quad_color: Color_drop_quad::id(),
            drop_quad: Background::default().with_draw_depth(3.),

            splitters: Elements::new(),
            tab_controls: Elements::new(),
            drop_quad_view: View::default().with_is_overlay(true),
            _close_tab: None,
            _drag_move: None,
            _drag_end: None,
            _tab_select: None,
            //_tweening_quad: None
        }
    }
}

impl<TItem> Dock<TItem>
where
    TItem: Clone,
{
    fn recur_remove_tab(
        dock_walk: &mut DockItem<TItem>,
        control_id: usize,
        tab_id: usize,
        counter: &mut usize,
        clone: bool,
        select_previous: bool,
    ) -> Option<DockTab<TItem>>
    where
        TItem: Clone,
    {
        match dock_walk {
            //DockItem::Single(_) => {},
            DockItem::TabControl { tabs, current, previous } => {
                let id = *counter;
                *counter += 1;
                if id == control_id {
                    if tab_id >= tabs.len() {
                        return None;
                    }
                    if clone && tabs[tab_id].closeable {
                        return Some(tabs[tab_id].clone());
                    } else {
                        // this select the previous tab.
                        if select_previous && *previous != *current && *previous < tabs.len() - 1 {
                            *current = *previous;
                        } else if *current >= 1 && *current == tabs.len() - 1 {
                            *current -= 1;
                        }
                        return Some(tabs.remove(tab_id));
                    }
                }
            }
            DockItem::Splitter { first, last, .. } => {
                *counter += 1;
                let left = Self::recur_remove_tab(first, control_id, tab_id, counter, clone, select_previous);
                if left.is_some() {
                    return left;
                }
                let right = Self::recur_remove_tab(last, control_id, tab_id, counter, clone, select_previous);
                if right.is_some() {
                    return right;
                }
            }
        }
        None
    }

    fn recur_collapse_empty(dock_walk: &mut DockItem<TItem>) -> bool
    where
        TItem: Clone,
    {
        match dock_walk {
            //DockItem::Single(_) => {},
            DockItem::TabControl { tabs, .. } => return tabs.is_empty(),
            DockItem::Splitter { first, last, .. } => {
                let rem_first = Self::recur_collapse_empty(first);
                let rem_last = Self::recur_collapse_empty(last);
                if rem_first && rem_last {
                    return true;
                }
                if rem_first {
                    *dock_walk = *last.clone();
                } else if rem_last {
                    *dock_walk = *first.clone();
                }
            }
        }
        false
    }

    fn recur_split_dock(
        dock_walk: &mut DockItem<TItem>,
        items: &[DockTab<TItem>],
        control_id: usize,
        kind: &DockDropKind,
        counter: &mut usize,
    ) -> Option<DockTabIdent>
    where
        TItem: Clone,
    {
        match dock_walk {
            //DockItem::Single(_) => {},
            DockItem::TabControl { tabs, previous: _, current } => {
                let id = *counter;
                *counter += 1;
                if id == control_id {
                    match kind {
                        DockDropKind::Tab(id) => {
                            let mut idc = *id;
                            for item in items {
                                tabs.insert(idc, item.clone());
                                idc += 1;
                            }
                            *current = idc - 1;
                            return Some(DockTabIdent { tab_control_id: control_id, tab_id: *current });
                        }
                        DockDropKind::Left => {
                            *dock_walk = DockItem::Splitter {
                                align: SplitterAlign::Weighted,
                                pos: 0.5,
                                axis: Axis::Vertical,
                                last: Box::new(dock_walk.clone()),
                                first: Box::new(DockItem::TabControl { current: 0, previous: 0, tabs: items.to_owned() }),
                            };
                            return Some(DockTabIdent { tab_control_id: control_id + 1, tab_id: 0 });
                        }
                        DockDropKind::Right => {
                            *dock_walk = DockItem::Splitter {
                                align: SplitterAlign::Weighted,
                                pos: 0.5,
                                axis: Axis::Vertical,
                                first: Box::new(dock_walk.clone()),
                                last: Box::new(DockItem::TabControl { current: 0, previous: 0, tabs: items.to_owned() }),
                            };
                            return Some(DockTabIdent { tab_control_id: control_id + 2, tab_id: 0 });
                        }
                        DockDropKind::Top => {
                            *dock_walk = DockItem::Splitter {
                                align: SplitterAlign::Weighted,
                                pos: 0.5,
                                axis: Axis::Horizontal,
                                last: Box::new(dock_walk.clone()),
                                first: Box::new(DockItem::TabControl { current: 0, previous: 0, tabs: items.to_owned() }),
                            };
                            return Some(DockTabIdent { tab_control_id: control_id + 1, tab_id: 0 });
                        }
                        DockDropKind::Bottom => {
                            *dock_walk = DockItem::Splitter {
                                align: SplitterAlign::Weighted,
                                pos: 0.5,
                                axis: Axis::Horizontal,
                                first: Box::new(dock_walk.clone()),
                                last: Box::new(DockItem::TabControl { current: 0, previous: 0, tabs: items.to_owned() }),
                            };
                            return Some(DockTabIdent { tab_control_id: control_id + 2, tab_id: 0 });
                        }
                        DockDropKind::TabsView | DockDropKind::Center => {
                            *current = tabs.len() + items.len() - 1;
                            for item in items {
                                tabs.push(item.clone());
                            }
                            return Some(DockTabIdent { tab_control_id: control_id, tab_id: tabs.len() - 1 });
                        }
                    }
                }
            }
            DockItem::Splitter { first, last, .. } => {
                *counter += 1;
                if let Some(ret) = Self::recur_split_dock(first, items, control_id, kind, counter) {
                    return Some(ret);
                }
                if let Some(ret) = Self::recur_split_dock(last, items, control_id, kind, counter) {
                    return Some(ret);
                }
            }
        }
        None
    }

    fn get_drop_kind(pos: Vec2, drop_size: Vec2, tvr: Rect, cdr: Rect, tab_rects: Vec<Rect>) -> (DockDropKind, Rect) {
        // this is how the drop areas look
        //    |            Tab                |
        //    |-------------------------------|
        //    |      |     Top        |       |
        //    |      |----------------|       |
        //    |      |                |       |
        //    |      |                |       |
        //    | Left |    Center      | Right |
        //    |      |                |       |
        //    |      |                |       |
        //    |      |----------------|       |
        //    |      |    Bottom      |       |
        //    ---------------------------------

        if tvr.contains(pos) {
            for (id, tr) in tab_rects.iter().enumerate() {
                if tr.contains(pos) {
                    return (DockDropKind::Tab(id), *tr);
                }
            }
            return (DockDropKind::TabsView, tvr);
        }
        if pos.y < cdr.pos.y + drop_size.y {
            return (DockDropKind::Top, Rect { pos: vec2(cdr.pos.x, cdr.pos.y), size: vec2(cdr.size.x, 0.5 * cdr.size.y) });
        }
        if pos.y > cdr.pos.y + cdr.size.y - drop_size.y {
            return (
                DockDropKind::Bottom,
                Rect { pos: vec2(cdr.pos.x, cdr.pos.y + 0.5 * cdr.size.y), size: vec2(cdr.size.x, 0.5 * cdr.size.y) },
            );
        }
        if pos.x < cdr.pos.x + drop_size.x {
            return (DockDropKind::Left, Rect { pos: vec2(cdr.pos.x, cdr.pos.y), size: vec2(0.5 * cdr.size.x, cdr.size.y) });
        }
        if pos.x > cdr.pos.x + cdr.size.x - drop_size.x {
            return (
                DockDropKind::Right,
                Rect { pos: vec2(cdr.pos.x + 0.5 * cdr.size.x, cdr.pos.y), size: vec2(0.5 * cdr.size.x, cdr.size.y) },
            );
        }
        (DockDropKind::Center, cdr)
    }

    pub fn dock_drag_out(&mut self, cx: &mut Cx) {
        self._drag_move = None;
        cx.request_draw();
    }

    pub fn dock_drag_move(&mut self, cx: &mut Cx, pe: PointerMoveEvent) {
        self._drag_move = Some(pe);
        cx.request_draw();
    }

    pub fn dock_drag_cancel(&mut self, cx: &mut Cx) {
        self._drag_move = None;
        cx.request_draw();
    }

    pub fn dock_drag_end(&mut self, _cx: &mut Cx, pe: PointerUpEvent, new_items: Vec<DockTab<TItem>>) {
        self._drag_move = None;
        self._drag_end = Some(DockDragEnd::NewItems { pe, items: new_items });
    }

    pub fn handle(&mut self, cx: &mut Cx, _event: &mut Event, dock_items: &mut DockItem<TItem>) -> DockEvent<TItem> {
        if let Some(close_tab) = &self._close_tab {
            let removed_tab =
                Self::recur_remove_tab(dock_items, close_tab.tab_control_id, close_tab.tab_id, &mut 0, false, false);
            Self::recur_collapse_empty(dock_items);
            cx.request_draw();
            self._close_tab = None;
            return DockEvent::DockTabClosed(removed_tab.unwrap().item);
        }
        if let Some(drag_end) = self._drag_end.clone() {
            self._drag_end = None;
            let mut tab_clone_ident = None;
            let pe = match &drag_end {
                DockDragEnd::OldTab { pe, .. } => pe,
                DockDragEnd::NewItems { pe, .. } => pe,
            };
            for (target_id, tab_control) in self.tab_controls.enumerate() {
                let cdr = tab_control.get_content_drop_rect(cx);
                let tvr = tab_control.get_tabs_view_rect(cx);
                if tvr.contains(pe.abs) || cdr.contains(pe.abs) {
                    // we might got dropped elsewhere
                    // ok now, we ask the tab_controls rect
                    let tab_rects = tab_control.get_tab_rects(cx);
                    let (kind, _rect) = Self::get_drop_kind(pe.abs, self.drop_size, tvr, cdr, tab_rects);

                    // alright our drag_end is an enum
                    // its either a previous tabs index
                    // or its a new Item
                    // we have a kind!
                    let mut do_tab_clone = false;
                    let items = match &drag_end {
                        DockDragEnd::OldTab { ident, .. } => {
                            if pe.modifiers.control || pe.modifiers.logo {
                                do_tab_clone = true;
                            }
                            let item = Self::recur_remove_tab(
                                dock_items,
                                ident.tab_control_id,
                                ident.tab_id,
                                &mut 0,
                                do_tab_clone,
                                true,
                            );
                            if let Some(item) = item {
                                if !item.closeable {
                                    do_tab_clone = false;
                                }
                                vec![item]
                            } else {
                                vec![]
                            }
                        }
                        DockDragEnd::NewItems { items, .. } => items.clone(),
                    };
                    // alright we have a kind.
                    if !items.is_empty() {
                        let new_ident = Self::recur_split_dock(dock_items, &items, *target_id, &kind, &mut 0);
                        if do_tab_clone {
                            tab_clone_ident = new_ident;
                        }
                    };
                }
            }
            Self::recur_collapse_empty(dock_items);
            cx.request_draw();
            //Self::recur_debug_dock(self.dock_items.as_mut().unwrap(), &mut 0, 0);
            if let Some(ident) = tab_clone_ident {
                return DockEvent::DockTabCloned { tab_control_id: ident.tab_control_id, tab_id: ident.tab_id };
            }
            return DockEvent::DockChanged;
        };
        // ok we need to pull out the TItem from our dockpanel
        DockEvent::None
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        // lets draw our hover layer if need be
        if let Some(pe) = &self._drag_move {
            cx.begin_absolute_box();
            self.drop_quad_view.begin_view(cx, LayoutSize::FILL);

            //let mut found_drop_zone = false;
            for (_id, tab_control) in self.tab_controls.enumerate() {
                let cdr = tab_control.get_content_drop_rect(cx);
                let tvr = tab_control.get_tabs_view_rect(cx);
                if tvr.contains(pe.abs) || cdr.contains(pe.abs) {
                    let tab_rects = tab_control.get_tab_rects(cx);
                    let (_kind, rect) = Self::get_drop_kind(pe.abs, self.drop_size, tvr, cdr, tab_rects);

                    //found_drop_zone = true;
                    self.drop_quad.draw(cx, rect.translate(cx.get_box_origin()), vec4(0.67, 0.67, 0.67, 0.8));
                }
            }
            //if !found_drop_zone {
            //    self._tweening_quad = None;
            //  }
            self.drop_quad_view.end_view(cx);
            cx.end_absolute_box();
        }
    }

    pub fn walker<'a>(&'a mut self, dock_items: &'a mut DockItem<TItem>) -> DockWalker<'a, TItem> {
        let stack = vec![DockWalkStack { counter: 0, uid: 0, item: dock_items }];
        self.splitters.begin_draw();
        self.tab_controls.begin_draw();
        DockWalker {
            walk_uid: 0,
            stack,
            splitters: &mut self.splitters,
            tab_controls: &mut self.tab_controls,
            _drag_move: &mut self._drag_move,
            _drag_end: &mut self._drag_end,
            _close_tab: &mut self._close_tab,
            _tab_select: &mut self._tab_select,
        }
    }
}

/*
fn recur_debug_dock(dock_walk:&mut DockItem<TItem>, counter:&mut usize, depth:usize)
where TItem: Clone
{
let mut indent = String::new();
for i in 0..depth{indent.push_str(" ")}
match dock_walk{
DockItem::Single(item)=>{},
DockItem::TabControl{tabs,..}=>{
let id = *counter;
*counter += 1;
println!("{}TabControl {}", indent, id);
for (id,tab) in tabs.iter().enumerate(){
println!("{} Tab{} {}", indent, id, tab.title);
}
},
DockItem::Splitter{first,last,..}=>{
let id = *counter;
*counter += 1;
println!("{}Splitter {}", indent, id);
Self::recur_debug_dock(first, counter, depth + 1);
Self::recur_debug_dock(last, counter, depth + 1);
}
}
}*/

/// TODO(JP): This seems like an overly complicated abstraction. For now I removed
/// it everywhere but kept it around in Dock.
///
/// Original comment:
/// These UI Element containers are the key to automating lifecycle mgmt
/// get_draw constructs items that don't exist yet,
/// and stores a redraw id. this is used by enumerate.
/// If an item is not 'get_draw'ed in a draw pass
/// it will be skipped by enumerate/iter.
/// However its not destructed and can be get-draw'ed
/// in another drawpass.
/// if you want to destruct items that werent get_draw'ed
/// call sweep on the elements collection.
/// If however you can also have 0 items in the collection,
/// You HAVE to use mark for sweep to work, since get auto-marks the collection
/// Redraw is incremental so there isn't a global 'redraw id' thats
/// always the same.
/// The idea is to use get_draw in a draw function
/// and use the iter/enumerate/get functions in the event handle code
/// This does not work for single item Element though
/// Keep a redraw ID with each element
/// to make iterating only 'redrawn in last pass' items possible
#[derive(Clone, Default)]
struct Elements<ID, T>
where
    ID: std::cmp::Ord + std::hash::Hash,
{
    element_list: Vec<ID>,
    element_map: HashMap<ID, ElementsRedraw<T>>,
    redraw_id: u64,
    marked_begin_draw: bool,
}

/// See [`Elements`].
#[derive(Clone, Default)]
struct ElementsRedraw<T> {
    redraw_id: u64,
    item: T,
}

/// See [`Elements`].
struct ElementsIterator<'a, ID, T>
where
    ID: std::cmp::Ord + std::hash::Hash,
{
    elements: &'a mut Elements<ID, T>,
    counter: usize,
}

impl<'a, ID, T> Iterator for ElementsIterator<'a, ID, T>
where
    ID: std::cmp::Ord + std::hash::Hash + Clone,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.counter >= self.elements.element_list.len() {
                return None;
            }
            let element_id = &self.elements.element_list[self.counter];
            let element = self.elements.element_map.get_mut(element_id).unwrap();
            self.counter += 1;
            if element.redraw_id == self.elements.redraw_id {
                return Some(unsafe { std::mem::transmute(&mut element.item) });
            }
        }
    }
}

/// See [`Elements`].
struct ElementsIteratorNamed<'a, ID, T>
where
    ID: std::cmp::Ord + std::hash::Hash,
{
    elements: &'a mut Elements<ID, T>,
    counter: usize,
}

impl<'a, ID, T> ElementsIteratorNamed<'a, ID, T>
where
    ID: std::cmp::Ord + std::hash::Hash,
{
    fn new(elements: &'a mut Elements<ID, T>) -> Self {
        ElementsIteratorNamed { elements, counter: 0 }
    }
}

impl<'a, ID, T> Iterator for ElementsIteratorNamed<'a, ID, T>
where
    ID: std::cmp::Ord + std::hash::Hash + Clone,
{
    type Item = (&'a ID, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.counter >= self.elements.element_list.len() {
                return None;
            }
            let element_id = &mut self.elements.element_list[self.counter];
            let element = self.elements.element_map.get_mut(element_id).unwrap();
            self.counter += 1;
            if element.redraw_id == self.elements.redraw_id {
                return Some((unsafe { std::mem::transmute(element_id) }, unsafe { std::mem::transmute(&mut element.item) }));
            }
        }
    }
}

impl<ID, T> Elements<ID, T>
where
    ID: std::cmp::Ord + std::hash::Hash + Clone,
{
    fn new() -> Elements<ID, T> {
        Elements::<ID, T> { redraw_id: 0, element_list: Vec::new(), element_map: HashMap::new(), marked_begin_draw: false }
    }

    // enumerate the set of 'last drawn' items
    fn enumerate(&mut self) -> ElementsIteratorNamed<ID, T> {
        ElementsIteratorNamed::new(self)
    }

    // gets a particular item. Returns None when not created (yet)
    fn get_mut(&mut self, index: ID) -> Option<&mut T> {
        let elem = self.element_map.get_mut(&index);
        if let Some(elem) = elem {
            Some(&mut elem.item)
        } else {
            None
        }
    }

    fn begin_draw(&mut self) {
        self.marked_begin_draw = true;
    }

    fn get_draw<F>(&mut self, index: ID, mut insert_callback: F) -> &mut T
    where
        F: FnMut() -> T,
    {
        if self.marked_begin_draw {
            self.marked_begin_draw = false;
            self.redraw_id += 1;
        }
        let element_list = &mut self.element_list;
        let redraw_id = self.redraw_id;
        let redraw = self.element_map.entry(index.clone()).or_insert_with(|| {
            element_list.push(index);
            let elem = insert_callback();
            ElementsRedraw { redraw_id, item: elem }
        });
        redraw.redraw_id = redraw_id;
        &mut redraw.item
    }
}
