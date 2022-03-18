use crate::listanims::*;
use zaplib::*;
use zaplib_components::*;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct FileTreeFillerIns {
    base: QuadIns,
    color: Vec4,
    line_vec: Vec2,
    anim_pos: f32,
}

impl FileTreeFillerIns {
    fn draw_quad_abs(&self, cx: &mut Cx, rect: Rect) {
        cx.add_instances(&SHADER, &[Self { base: QuadIns::from_rect(rect).with_draw_depth(0.2), color: COLOR_FILLER, ..*self }]);
    }
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance color: vec4;
            instance line_vec: vec2;
            instance anim_pos: float;
            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);
                let w = rect_size.x;
                let h = rect_size.y;
                if anim_pos< -0.5 {
                    df.move_to(vec2(0.5 * w, line_vec.x * h));
                    df.line_to(vec2(0.5 * w, line_vec.y * h));
                    return df.stroke(color * 0.5, 1.);
                }
                else { // its a folder
                    df.box(vec2(0. * w, 0.35 * h), vec2(0.87 * w, 0.39 * h), 0.75);
                    df.box(vec2(0. * w, 0.28 * h), vec2(0.5 * w, 0.3 * h), 1.);
                    df.union();
                    // ok so.
                    return df.fill(color);
                }
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

pub struct FileTree {
    pub view: ScrollView,
    drag_view: View,
    _drag_move: Option<PointerMoveEvent>,
    pub root_node: FileNode,
    drag_bg: Background,

    node_bg: Background,

    node_layout_size: LayoutSize,
    row_height: f32,
    color_tree_folder: Vec4,
    color_tree_file: Vec4,

    event_id: u64,
}

#[derive(Clone, PartialEq)]
pub enum FileTreeEvent {
    None,
    DragMove { pe: PointerMoveEvent, paths: Vec<String> },
    DragCancel,
    DragEnd { pe: PointerUpEvent, paths: Vec<String> },
    SelectFile { path: String },
    SelectFolder { path: String },
}

const FILLER_WALK: LayoutSize = LayoutSize { width: Width::Fix(10.0), height: Height::Fill };
const FILLER_PADDING: Padding = Padding { l: 1., t: 0., r: 4., b: 0. };

const NODE_PADDING: Padding = Padding { l: 5., t: 0., r: 0., b: 1. };

const TEXT_STYLE_LABEL: TextStyle = TextStyle { top_drop: 1.3, ..TEXT_STYLE_NORMAL };

const COLOR_TREE_FOLDER: Vec4 = vec4(1.0, 1.0, 1.0, 1.0);
const COLOR_TREE_FILE: Vec4 = vec4(157.0 / 255.0, 157.0 / 255.0, 157.0 / 255.0, 1.0);
const COLOR_FILLER: Vec4 = vec4(127.0 / 255.0, 127.0 / 255.0, 127.0 / 255.0, 1.0);
const COLOR_DRAG_BG: Vec4 = vec4(17.0 / 255.0, 70.0 / 255.0, 110.0 / 255.0, 1.0);

impl FileTree {
    pub fn new() -> Self {
        Self {
            root_node: FileNode::Folder {
                name: "".to_string(),
                state: NodeState::Open,
                draw: None,
                folder: vec![FileNode::File { name: "loading...".to_string(), draw: None }],
            },

            drag_bg: Background::default().with_radius(2.),

            view: ScrollView::default().with_scroll_v(ScrollBarConfig::default().with_smoothing(0.15)),

            drag_view: View::default().with_is_overlay(true),

            _drag_move: None,

            node_bg: Background::default(),
            //node_layout: LayoutFileTreeNode::id(),
            node_layout_size: LayoutSize::default(),
            row_height: 0.,
            color_tree_folder: Vec4::default(),
            color_tree_file: Vec4::default(),

            event_id: 0,
        }
    }

    fn apply_style(&mut self) {
        let node_height = 20.;
        self.node_layout_size = LayoutSize::new(Width::Fill, Height::Fix(node_height));
        self.row_height = node_height;
        self.color_tree_folder = COLOR_TREE_FOLDER;
        self.color_tree_file = COLOR_TREE_FILE;
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

    fn get_marked_paths(root: &mut FileNode) -> Vec<String> {
        let mut paths = Vec::new();
        let mut file_walker = FileWalker::new(root);
        // make a path set of all marked items
        while let Some((_depth, _index, _len, node)) = file_walker.walk() {
            let node_draw = if let Some(node_draw) = node.get_draw() {
                node_draw
            } else {
                continue;
            };
            if node_draw.marked != 0 {
                paths.push(file_walker.current_path());
            }
        }
        paths
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> FileTreeEvent {
        self.event_id += 1;

        let mut file_walker = FileWalker::new(&mut self.root_node);
        let mut counter = 0;
        self.view.handle(cx, event);
        // todo, optimize this so events are not passed through 'all' of our tree elements
        // but filtered out somewhat based on a bounding rect
        let mut unmark_nodes = false;
        let mut drag_nodes = false;
        let mut drag_end: Option<PointerUpEvent> = None;
        let mut select_node = 0;
        while let Some((_depth, _index, _len, node)) = file_walker.walk() {
            // alright we haz a node. so now what.
            let is_filenode = if let FileNode::File { .. } = node { true } else { false };

            let node_draw = if let Some(node_draw) = node.get_draw() {
                node_draw
            } else {
                continue;
            };

            if node_draw.animator.handle(cx, event) {
                self.node_bg.set_area(node_draw.area);
                self.node_bg.set_color(cx, node_draw.animator.get_vec4(0));
            }

            match event.hits_pointer(cx, node_draw.component_id, node_draw.area.get_rect_for_first_instance(cx)) {
                Event::PointerDown(_pe) => {
                    // mark ourselves, unmark others
                    if is_filenode {
                        select_node = 1;
                    } else {
                        select_node = 2;
                    }
                    node_draw.marked = self.event_id;

                    unmark_nodes = true;
                    node_draw.animator.play_anim(cx, Self::get_over_anim(counter, node_draw.marked != 0));

                    if let FileNode::Folder { state, .. } = node {
                        *state = match state {
                            NodeState::Opening(fac) => NodeState::Closing(1.0 - *fac),
                            NodeState::Closing(fac) => NodeState::Opening(1.0 - *fac),
                            NodeState::Open => NodeState::Closing(1.0),
                            NodeState::Closed => NodeState::Opening(1.0),
                        };
                        // start the redraw loop
                        cx.request_draw();
                    }
                }
                Event::PointerUp(pe) => {
                    if self._drag_move.is_some() {
                        drag_end = Some(pe);
                        // we now have to do the drop....
                        cx.request_draw();
                        //self._drag_move = None;
                    }
                }
                Event::PointerMove(pe) => {
                    cx.set_down_mouse_cursor(MouseCursor::Hand);
                    if self._drag_move.is_none() {
                        if pe.move_distance() > 50. {
                            self._drag_move = Some(pe);
                            cx.request_draw();
                        }
                    } else {
                        self._drag_move = Some(pe);
                        cx.request_draw();
                    }
                    drag_nodes = true;
                }
                Event::PointerHover(pe) => {
                    cx.set_hover_mouse_cursor(MouseCursor::Hand);
                    match pe.hover_state {
                        HoverState::In => {
                            node_draw.animator.play_anim(cx, Self::get_over_anim(counter, node_draw.marked != 0));
                        }
                        HoverState::Out => {
                            node_draw.animator.play_anim(cx, Self::get_default_anim(counter, node_draw.marked != 0));
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
            counter += 1;
        }

        //unmark non selected nodes and also set even/odd animations to make sure its rendered properly
        if unmark_nodes {
            let mut file_walker = FileWalker::new(&mut self.root_node);
            let mut counter = 0;
            while let Some((_depth, _index, _len, node)) = file_walker.walk() {
                if let Some(node_draw) = node.get_draw() {
                    if node_draw.marked != self.event_id || node_draw.marked == 0 {
                        node_draw.marked = 0;
                        node_draw.animator.play_anim(cx, Self::get_default_anim(counter, false));
                    }
                }
                if !file_walker.current_closing() {
                    counter += 1;
                }
            }
        }
        if let Some(pe) = drag_end {
            self._drag_move = None;
            let paths = Self::get_marked_paths(&mut self.root_node);
            if !self.view.area().get_rect_for_first_instance(cx).unwrap_or_default().contains(pe.abs) {
                return FileTreeEvent::DragEnd { pe, paths };
            }
        }
        if drag_nodes {
            if let Some(pe) = &self._drag_move {
                // lets check if we are over our own filetree
                // ifso, we need to support moving files with directories
                let paths = Self::get_marked_paths(&mut self.root_node);
                if !self.view.area().get_rect_for_first_instance(cx).unwrap_or_default().contains(pe.abs) {
                    return FileTreeEvent::DragMove { pe: pe.clone(), paths };
                } else {
                    return FileTreeEvent::DragCancel;
                }
            }
        };
        if select_node != 0 {
            let mut file_walker = FileWalker::new(&mut self.root_node);
            while let Some((_depth, _index, _len, node)) = file_walker.walk() {
                let node_draw = if let Some(node_draw) = node.get_draw() {
                    node_draw
                } else {
                    continue;
                };
                if node_draw.marked != 0 {
                    if select_node == 1 {
                        return FileTreeEvent::SelectFile { path: file_walker.current_path() };
                    } else {
                        return FileTreeEvent::SelectFolder { path: file_walker.current_path() };
                    }
                }
            }
        }
        FileTreeEvent::None
    }

    fn walk_filler(cx: &mut Cx) -> Rect {
        cx.begin_padding_box(FILLER_PADDING);
        let rect = cx.add_box(FILLER_WALK);
        cx.end_padding_box();
        rect
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.view.begin_view(cx, LayoutSize::FILL);

        self.apply_style();

        let mut file_walker = FileWalker::new(&mut self.root_node);

        // lets draw the filetree
        let mut counter = 0;
        let mut scale_stack = Vec::new();
        let mut last_stack = Vec::new();
        scale_stack.push(1.0f64);

        let mut tree_text_props = TextInsProps { text_style: TEXT_STYLE_LABEL, draw_depth: 0.1, ..TextInsProps::DEFAULT };

        while let Some((depth, index, len, node)) = file_walker.walk() {
            let is_first = index == 0;
            let is_last = index == len - 1;

            while depth < scale_stack.len() {
                scale_stack.pop();
                last_stack.pop();
            }
            let scale = scale_stack[depth - 1];

            // lets store the bg area in the tree
            let node_draw = node.get_draw();
            if node_draw.is_none() {
                *node_draw = Some(NodeDraw::default())
            }
            let node_draw = node_draw.as_mut().unwrap();

            // using set area is necessary because we don't keep one instance
            // of the draw api with the item.
            self.node_bg.set_area(node_draw.area);

            node_draw.animator.draw(cx, Self::get_default_anim(counter, false));

            let mut node_layout_size = self.node_layout_size;
            node_layout_size.height = Height::Fix(self.row_height * scale as f32);

            self.node_bg.begin_draw(cx, node_layout_size.width, node_layout_size.height, node_draw.animator.get_vec4(0));
            cx.begin_padding_box(NODE_PADDING);

            node_draw.area = self.node_bg.area();

            let is_marked = node_draw.marked != 0;

            let mut filler = FileTreeFillerIns::default();
            for i in 0..(depth - 1) {
                if i == depth - 2 {
                    // our own thread.
                    if is_last {
                        if is_first {
                            //line_vec
                            filler.line_vec = vec2(0.3, 0.7);
                        } else {
                            //line_vec
                            filler.line_vec = vec2(-0.2, 0.7);
                        }
                    } else if is_first {
                        //line_vec
                        filler.line_vec = vec2(-0.3, 1.2)
                    } else {
                        //line_vec
                        filler.line_vec = vec2(-0.2, 1.2);
                    }
                    //anim_pos
                    filler.anim_pos = -1.;
                    let rect = Self::walk_filler(cx);
                    filler.draw_quad_abs(cx, rect);
                } else {
                    let here_last = if last_stack.len() > 1 { last_stack[i + 1] } else { false };
                    if here_last {
                        Self::walk_filler(cx);
                    } else {
                        filler.line_vec = vec2(-0.2, 1.2);
                        filler.anim_pos = -1.;
                        let rect = Self::walk_filler(cx);
                        filler.draw_quad_abs(cx, rect);
                    }
                }
            }
            //self.item_draw.filler.z = 0.;
            //self.item_draw.tree_text.z = 0.;
            //self.item_draw.tree_text.font_size = self.font_size;
            tree_text_props.font_scale = scale as f32;
            match node {
                FileNode::Folder { name, state, .. } => {
                    // draw the folder icon
                    filler.line_vec = vec2(0., 0.);
                    filler.anim_pos = 1.;
                    let rect = Self::walk_filler(cx);
                    filler.draw_quad_abs(cx, rect);
                    cx.begin_row(self.node_layout_size.width, self.node_layout_size.height);
                    cx.begin_center_y_align();
                    tree_text_props.color = self.color_tree_folder;
                    let wleft = cx.get_width_left() - 10.;
                    tree_text_props.wrapping = Wrapping::Ellipsis(wleft);
                    TextIns::draw_walk(cx, name, &tree_text_props);
                    cx.end_center_y_align();
                    cx.end_row();

                    let (new_scale, new_state) = match state {
                        NodeState::Opening(fac) => {
                            cx.request_draw();
                            if *fac < 0.001 {
                                (1.0, NodeState::Open)
                            } else {
                                (1.0 - *fac, NodeState::Opening(*fac * 0.6))
                            }
                        }
                        NodeState::Closing(fac) => {
                            cx.request_draw();
                            if *fac < 0.001 {
                                (0.0, NodeState::Closed)
                            } else {
                                (*fac, NodeState::Closing(*fac * 0.6))
                            }
                        }
                        NodeState::Open => (1.0, NodeState::Open),
                        NodeState::Closed => (1.0, NodeState::Closed),
                    };
                    *state = new_state;
                    last_stack.push(is_last);
                    scale_stack.push(scale * new_scale);
                }
                FileNode::File { name, .. } => {
                    cx.begin_row(self.node_layout_size.width, self.node_layout_size.height);
                    cx.begin_center_y_align();
                    let wleft = cx.get_width_left() - 10.;
                    tree_text_props.wrapping = Wrapping::Ellipsis(wleft);
                    tree_text_props.color = if is_marked { self.color_tree_folder } else { self.color_tree_file };
                    TextIns::draw_walk(cx, name, &tree_text_props);
                    cx.end_center_y_align();
                    cx.end_row();
                }
            }
            cx.end_padding_box();
            self.node_bg.end_draw(cx);

            // if any of the parents is closing, don't count alternating lines
            if !file_walker.current_closing() {
                counter += 1;
            }
        }

        // draw filler nodes
        if self.row_height > 0. {
            let view_total = cx.get_box_bounds();
            let rect_now = cx.get_box_rect();
            let mut y = view_total.y;
            while y < rect_now.size.y {
                self.node_bg.begin_draw(
                    cx,
                    Width::Fill,
                    Height::Fix((rect_now.size.y - y).min(self.row_height)),
                    if counter & 1 == 0 { LIST_ANIMS_COLOR_BG_EVEN } else { LIST_ANIMS_COLOR_BG_ODD },
                );
                self.node_bg.end_draw(cx);
                y += self.row_height;
                counter += 1;
            }
        }

        // draw the drag item overlay layer if need be
        if let Some(mv) = &self._drag_move {
            cx.begin_absolute_box();
            self.drag_view.begin_view(cx, LayoutSize::FILL);
            cx.begin_padding_box(Padding { l: mv.abs.x + 5., t: mv.abs.y + 5., r: 0., b: 0. });

            let mut file_walker = FileWalker::new(&mut self.root_node);
            while let Some((_depth, _index, _len, node)) = file_walker.walk() {
                let node_draw = if let Some(node_draw) = node.get_draw() {
                    node_draw
                } else {
                    continue;
                };
                if node_draw.marked != 0 {
                    //self.drag_bg.z = 10.0;
                    //self.item_draw.tree_text.z = 10.0;
                    self.drag_bg.begin_draw(cx, Width::Compute, Height::Compute, COLOR_DRAG_BG);
                    cx.begin_padding_box(Padding::all(5.));
                    tree_text_props.color = COLOR_TREE_FOLDER;
                    let name = match node {
                        FileNode::Folder { name, .. } => name,
                        FileNode::File { name, .. } => name,
                    };
                    TextIns::draw_walk(cx, name, &tree_text_props);
                    cx.end_padding_box();
                    self.drag_bg.end_draw(cx);
                }
            }

            cx.end_padding_box();
            self.drag_view.end_view(cx);
            cx.end_absolute_box();
        }

        ScrollShadow::draw_shadow_top(cx, 0.25);

        self.view.end_view(cx);
    }
}

#[derive(Clone)]
pub enum NodeState {
    Open,
    Opening(f64),
    Closing(f64),
    Closed,
}

#[derive(Default)]
pub struct NodeDraw {
    component_id: ComponentId,
    area: Area,
    animator: Animator,
    marked: u64,
}

pub enum FileNode {
    File { name: String, draw: Option<NodeDraw> },
    Folder { name: String, draw: Option<NodeDraw>, state: NodeState, folder: Vec<FileNode> },
}

impl FileNode {
    fn get_draw<'a>(&'a mut self) -> &'a mut Option<NodeDraw> {
        match self {
            FileNode::File { draw, .. } => draw,
            FileNode::Folder { draw, .. } => draw,
        }
    }

    fn name(&self) -> String {
        match self {
            FileNode::File { name, .. } => name.clone(),
            FileNode::Folder { name, .. } => name.clone(),
        }
    }
}

struct StackEntry<'a> {
    counter: usize,
    index: usize,
    len: usize,
    closing: bool,
    node: &'a mut FileNode,
}

pub struct FileWalker<'a> {
    stack: Vec<StackEntry<'a>>,
}

// this flattens out recursion into an iterator. unfortunately needs unsafe. come on. thats not nice
impl<'a> FileWalker<'a> {
    pub fn new(root_node: &'a mut FileNode) -> FileWalker<'a> {
        FileWalker { stack: vec![StackEntry { counter: 1, closing: false, index: 0, len: 0, node: root_node }] }
    }

    pub fn current_path(&self) -> String {
        // the current stack top returned as path
        let mut path = String::new();
        for i in 0..self.stack.len() {
            if i > 1 {
                path.push('/');
            }
            path.push_str(&self.stack[i].node.name());
        }
        path
    }

    pub fn current_closing(&self) -> bool {
        if let Some(stack_top) = self.stack.last() {
            stack_top.closing
        } else {
            false
        }
    }

    pub fn walk(&mut self) -> Option<(usize, usize, usize, &mut FileNode)> {
        // lets get the current item on the stack
        let stack_len = self.stack.len();
        let push_or_pop = if let Some(stack_top) = self.stack.last_mut() {
            // return item 'count'
            match stack_top.node {
                FileNode::File { .. } => {
                    stack_top.counter += 1;
                    if stack_top.counter == 1 {
                        return Some((stack_len - 1, stack_top.index, stack_top.len, unsafe {
                            std::mem::transmute(&mut *stack_top.node)
                        }));
                    }
                    None // pop stack
                }
                FileNode::Folder { folder, state, .. } => {
                    stack_top.counter += 1;
                    if stack_top.counter == 1 {
                        // return self
                        return Some((stack_len - 1, stack_top.index, stack_top.len, unsafe {
                            std::mem::transmute(&mut *stack_top.node)
                        }));
                    } else {
                        let child_index = stack_top.counter - 2;
                        let opened = if let NodeState::Closed = state { false } else { true };
                        let closing = if let NodeState::Closing(_) = state { true } else { stack_top.closing };
                        if opened && child_index < folder.len() {
                            // child on stack
                            Some(StackEntry {
                                counter: 0,
                                closing,
                                index: child_index,
                                len: folder.len(),
                                node: unsafe { std::mem::transmute(&mut folder[child_index]) },
                            })
                        } else {
                            None // pop stack
                        }
                    }
                }
            }
        } else {
            None
        };
        if let Some(item) = push_or_pop {
            self.stack.push(item);
            return self.walk();
        } else if !self.stack.is_empty() {
            self.stack.pop();
            return self.walk();
        }
        None
    }
}
