//use syn::Type;
use crate::buildmanager::*;
use crate::fileeditor::*;
use crate::filepanel::*;
use crate::filetree::*;
use crate::homepage::*;
use crate::itemdisplay::*;
use crate::keyboard::*;
use crate::loglist::*;
use crate::makepadstorage::*;
use crate::searchresults::*;
use crate::worldview::WorldView;
use std::collections::HashMap;
use zaplib::*;
use zaplib_components::*;

#[derive(Debug, Clone)]
pub enum Panel {
    LogList,
    SearchResults,
    ItemDisplay,
    Keyboard,
    FileTree,
    FileEditorTarget,
    FileEditor { path: String, scroll_pos: Vec2, editor_id: u64 },
    WorldSelect,
    WorldView,
}

pub struct ZapWindow {
    desktop_window: DesktopWindow,
    pub file_panel: FilePanel,
    home_page: HomePage,
    item_display: ItemDisplay,
    log_list: LogList,
    search_results: SearchResults,
    keyboard: Keyboard,
    file_editors: FileEditors,
    world_view: WorldView,
    dock: Dock<Panel>,
    dock_items: DockItem<Panel>,
}

impl ZapWindow {
    pub fn new(cx: &mut Cx) -> Self {
        Self {
            desktop_window: DesktopWindow::default().with_caption("Bigedit"),

            file_editors: FileEditors { editors: HashMap::new() },
            home_page: HomePage::new(cx),
            keyboard: Keyboard::new(),
            item_display: ItemDisplay::new(),
            log_list: LogList::new(),
            search_results: SearchResults::new(),
            file_panel: FilePanel::new(),
            world_view: WorldView::default(),
            dock: Dock::default(),
            dock_items: DockItem::Splitter {
                axis: Axis::Vertical,
                align: SplitterAlign::First,
                pos: 200.0,

                first: Box::new(DockItem::Splitter {
                    axis: Axis::Horizontal,
                    align: SplitterAlign::Last,
                    pos: 250.0,
                    first: Box::new(DockItem::TabControl {
                        current: 0,
                        previous: 0,
                        tabs: vec![
                            DockTab { closeable: false, title: "Files".to_string(), item: Panel::FileTree },
                            DockTab { closeable: false, title: "".to_string(), item: Panel::SearchResults },
                        ],
                    }),
                    last: Box::new(DockItem::TabControl {
                        current: 0,
                        previous: 0,
                        tabs: vec![DockTab { closeable: false, title: "WorldView".to_string(), item: Panel::WorldView }],
                    }),
                }),
                last: Box::new(DockItem::Splitter {
                    axis: Axis::Horizontal,
                    align: SplitterAlign::Last,
                    pos: 100.0,
                    first: Box::new(DockItem::TabControl {
                        current: 1,
                        previous: 0,
                        tabs: vec![
                            DockTab { closeable: false, title: "Edit".to_string(), item: Panel::FileEditorTarget },
                            DockTab {
                                closeable: true,
                                title: "treeworld.rs".to_string(),
                                item: Panel::FileEditor {
                                    path: "main/zaplib/zaplib/examples/example_bigedit/src/treeworld.rs".to_string(),
                                    scroll_pos: Vec2::default(),
                                    editor_id: 2,
                                },
                            },
                        ],
                    }),

                    last: Box::new(DockItem::Splitter {
                        axis: Axis::Vertical,
                        align: SplitterAlign::Last,
                        pos: 200.0,
                        first: Box::new(DockItem::TabControl {
                            current: 0,
                            previous: 0,
                            tabs: vec![DockTab { closeable: false, title: "Log".to_string(), item: Panel::LogList }],
                        }),
                        last: Box::new(DockItem::TabControl {
                            current: 0,
                            previous: 0,
                            tabs: vec![
                                DockTab { closeable: false, title: "WorldSelect".to_string(), item: Panel::WorldSelect },
                                DockTab { closeable: false, title: "Item".to_string(), item: Panel::ItemDisplay },
                                DockTab { closeable: false, title: "Keyboard".to_string(), item: Panel::Keyboard },
                            ],
                        }),
                    }),
                }),
            },
        }
    }

    pub fn handle(
        &mut self,
        cx: &mut Cx,
        event: &mut Event,
        window_index: usize,
        makepad_storage: &mut MakepadStorage,
        build_manager: &mut BuildManager,
    ) {
        match self.desktop_window.handle(cx, event) {
            DesktopWindowEvent::EventForOtherWindow => return,
            DesktopWindowEvent::WindowClosed => return,
            _ => (),
        }

        match event {
            Event::KeyDown(ke) => match ke.key_code {
                KeyCode::Backtick => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        if build_manager.active_builds.is_empty() {
                            // build_manager.restart_build(cx, makepad_storage);
                        }
                        let mut clear = true;
                        for ab in &build_manager.active_builds {
                            if ab.build_uid.is_some() {
                                clear = false;
                            }
                        }
                        if clear {
                            build_manager.tail_log_items = true;
                            build_manager.log_items.truncate(0);
                        }
                        // build_manager.artifact_run(makepad_storage);
                        self.show_log_tab(cx, window_index);
                    }
                }
                _ => (),
            },
            _ => (),
        }

        if self.search_results.handle_search_input(cx, event, &mut build_manager.search_index, makepad_storage) {
            self.show_search_tab(cx, window_index);
        }

        self.world_view.handle_world_view(cx, event);

        let dock_items = &mut self.dock_items;
        let mut dock_walker = self.dock.walker(dock_items);
        let mut file_tree_event = FileTreeEvent::None;
        //let mut text_editor_event = TextEditorEvent::None;
        let mut set_last_cursor = None;
        let mut do_search = None;
        let mut show_item_display_tab = false;
        let mut do_display_rust_file = None;

        while let Some((item, _dock_tab_ident)) = dock_walker.walk_handle_dock(cx, event) {
            match item {
                Panel::LogList => {
                    match self.log_list.handle(cx, event, makepad_storage, build_manager) {
                        LogListEvent::SelectLocMessage { loc_message, jump_to_offset } => {
                            // just make it open an editor
                            if !loc_message.path.is_empty() {
                                // ok so. lets lookup the path in our remap list
                                //println!("TRYING TO SELECT FILE {} ")
                                file_tree_event =
                                    FileTreeEvent::SelectFile { path: makepad_storage.remap_sync_path(&loc_message.path) };
                            }
                            self.item_display.display_message(cx, &loc_message);
                            set_last_cursor = Some((jump_to_offset, jump_to_offset));
                            show_item_display_tab = true;
                        }
                        LogListEvent::SelectMessages { items } => {
                            self.item_display.display_plain_text(cx, &items);
                            show_item_display_tab = true;
                        }
                        _ => (),
                    }
                }
                Panel::WorldView => {}
                Panel::WorldSelect => {
                    self.world_view.handle_world_select(cx, event);
                }
                Panel::ItemDisplay => {
                    self.item_display.handle(cx, event);
                }
                Panel::SearchResults => {
                    match self.search_results.handle_search_results(cx, event, &mut build_manager.search_index, makepad_storage) {
                        SearchResultEvent::DisplayFile { text_buffer_id, cursor } => {
                            set_last_cursor = Some(cursor);
                            do_display_rust_file = Some(text_buffer_id);
                        }
                        SearchResultEvent::OpenFile { text_buffer_id, cursor } => {
                            let path =
                                makepad_storage.text_buffer_id_to_path.get(&text_buffer_id).expect("Path not found").clone();
                            file_tree_event = FileTreeEvent::SelectFile { path };
                            set_last_cursor = Some(cursor);
                        }
                        _ => (),
                    }
                }
                Panel::Keyboard => {
                    self.keyboard.handle(cx, event, makepad_storage);
                }
                Panel::FileEditorTarget => {
                    self.home_page.handle(cx, event);
                }
                Panel::FileTree => {
                    file_tree_event = self.file_panel.handle(cx, event);
                }
                Panel::FileEditor { path, scroll_pos: _, editor_id } => {
                    if let Some(file_editor) = &mut self.file_editors.editors.get_mut(editor_id) {
                        let mtb = makepad_storage.text_buffer_from_path(cx, path);

                        match file_editor.handle(cx, event, mtb, Some(&mut build_manager.search_index)) {
                            TextEditorEvent::Search(search) => {
                                do_search = Some((Some(search), mtb.text_buffer_id, true, false));
                            }
                            TextEditorEvent::Decl(search) => {
                                do_search = Some((Some(search), mtb.text_buffer_id, false, false));
                            }
                            TextEditorEvent::Escape => {
                                do_search = Some((Some("".to_string()), mtb.text_buffer_id, false, true));
                            }
                            TextEditorEvent::Change => {
                                // lets post a new file to our local thing
                                // and send over the cursor change
                                do_search = Some((None, MakepadTextBufferId(0), false, false));
                            }
                            TextEditorEvent::LagChange => {
                                makepad_storage.text_buffer_file_write(cx, path);
                                if makepad_storage.settings.build_on_save {
                                    // build_manager.restart_build(cx, makepad_storage);
                                }
                            }
                            TextEditorEvent::CursorMove => {
                                // lets send over the cursor set.
                            }
                            _ => (),
                        }
                    }
                }
            }
        }

        if let Some((search, first_tbid, focus, escape)) = do_search {
            if let Some(search) = search {
                self.search_results.set_search_input_value(cx, &search, first_tbid, focus);
            }
            let first_result = self.search_results.do_search(cx, &mut build_manager.search_index, makepad_storage);
            if escape {
                self.show_files_tab(cx, window_index);
            }
            if focus {
                self.show_search_tab(cx, window_index);
            } else if let Some((tbid, cursor)) = first_result {
                set_last_cursor = Some(cursor);
                do_display_rust_file = Some(tbid);
            }
        }

        if show_item_display_tab {
            self.show_item_display_tab(cx, window_index);
        }

        if let Some(tbid) = do_display_rust_file {
            let path = makepad_storage.text_buffer_id_to_path.get(&tbid).unwrap();
            if self.open_preview_editor_tab(cx, window_index, path, set_last_cursor) {
                self.ensure_unique_tab_title_for_file_editors(cx, window_index);
            }
        }

        match file_tree_event {
            FileTreeEvent::DragMove { pe, .. } => {
                self.dock.dock_drag_move(cx, pe);
            }
            FileTreeEvent::DragCancel => {
                self.dock.dock_drag_cancel(cx);
            }
            FileTreeEvent::DragEnd { pe, paths } => {
                let mut tabs = Vec::new();
                for path in paths {
                    // find a free editor id
                    tabs.push(self.new_file_editor_tab(cx, &path, None, false, true));
                }
                self.dock.dock_drag_end(cx, pe, tabs);
                self.ensure_unique_tab_title_for_file_editors(cx, window_index);
            }
            FileTreeEvent::SelectFile { path } => {
                // search for the tabcontrol with the maximum amount of editors
                if self.focus_or_new_editor(cx, window_index, &path, set_last_cursor) {
                    self.ensure_unique_tab_title_for_file_editors(cx, window_index);
                }
            }
            _ => {}
        }

        let dock_items = &mut self.dock_items;
        match self.dock.handle(cx, event, dock_items) {
            // TODO(Paras): We added the _item here, and is ignored because Makepad does not clean up [`Self::file_editors`]
            // when you close tabs. This is technically a memory leak and may be worth handling in the future.
            DockEvent::DockTabClosed(_item) => {
                self.ensure_unique_tab_title_for_file_editors(cx, window_index);
            }
            DockEvent::DockTabCloned { tab_control_id, tab_id } => {
                // lets change up our editor_id
                let max_id = self.file_editors.highest_file_editor_id();
                let mut dock_walker = self.dock.walker(dock_items);
                while let Some((ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
                    match dock_item {
                        DockItem::TabControl { tabs, .. } => {
                            if ctrl_id == tab_control_id {
                                if let Some(tab) = tabs.get_mut(tab_id) {
                                    match &mut tab.item {
                                        Panel::FileEditor { editor_id, .. } => {
                                            // we need to make a new editor_id here.
                                            *editor_id = max_id + 1;
                                            break;
                                            // and now it needs to scroll the new one....
                                        }
                                        _ => (),
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    pub fn draw(
        &mut self,
        cx: &mut Cx,
        menu: &Menu,
        _window_index: usize,
        makepad_storage: &mut MakepadStorage,
        build_manager: &mut BuildManager,
    ) {
        self.desktop_window.begin_draw(cx, Some(menu));

        self.dock.draw(cx);

        let dock_items = &mut self.dock_items;
        let mut dock_walker = self.dock.walker(dock_items);
        let file_panel = &mut self.file_panel;
        let search_results = &mut self.search_results;
        let item_display = &mut self.item_display;
        while let Some(item) = dock_walker.walk_draw_dock(cx, |cx, tab_control, tab, selected| {
            // this draws the tabs, so we can customize it
            match tab.item {
                Panel::FileTree => {
                    let tab = tab_control.get_draw_tab(cx, &tab.title, selected, tab.closeable);
                    tab.begin_tab(cx);
                    file_panel.draw_tab(cx);
                    tab.end_tab(cx);
                }
                Panel::SearchResults => {
                    let tab = tab_control.get_draw_tab(cx, &tab.title, selected, tab.closeable);
                    tab.begin_tab(cx);
                    search_results.draw_search_result_tab(cx, &build_manager.search_index);
                    tab.end_tab(cx);
                }
                _ => tab_control.draw_tab(cx, &tab.title, selected, tab.closeable),
            }
        }) {
            match item {
                Panel::WorldView => {
                    self.world_view.draw_world_view_2d(cx);
                }
                Panel::WorldSelect => {
                    self.world_view.draw_world_select(cx);
                }
                Panel::LogList => {
                    self.log_list.draw(cx, build_manager);
                }
                Panel::SearchResults => {
                    search_results.draw_search_results(cx, makepad_storage);
                }
                Panel::ItemDisplay => {
                    item_display.draw(cx);
                }
                Panel::Keyboard => {
                    self.keyboard.draw(cx);
                }
                Panel::FileEditorTarget => {
                    self.home_page.draw(cx);
                }
                Panel::FileTree => {
                    file_panel.draw(cx);
                }
                Panel::FileEditor { path, scroll_pos, editor_id } => {
                    let text_buffer = makepad_storage.text_buffer_from_path(cx, path);
                    let (file_editor, is_new) = self.file_editors.get_file_editor_for_path(path, *editor_id);
                    if is_new {
                        file_editor.set_scroll_pos_on_load(*scroll_pos);
                    }
                    file_editor.draw(cx, text_buffer, &mut build_manager.search_index);
                }
            }
        }

        if self.desktop_window.window.xr_is_presenting(cx) {
            self.world_view.draw_world_view_3d(cx);
        }
        self.desktop_window.end_draw(cx);
    }

    fn ensure_unique_tab_title_for_file_editors(&mut self, cx: &mut Cx, _window_index: usize) {
        // we walk through the dock collecting tab titles, if we run into a collision
        // we need to find the shortest uniqueness
        let mut collisions: HashMap<String, Vec<(String, usize, usize)>> = HashMap::new();
        let dock_items = &mut self.dock_items;
        // enumerate dockspace and collect all names
        let mut dock_walker = self.dock.walker(dock_items);
        while let Some((ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
            if let DockItem::TabControl { tabs, .. } = dock_item {
                for (id, tab) in tabs.iter_mut().enumerate() {
                    if let Panel::FileEditor { path, .. } = &tab.item {
                        let title = path_file_name(path);
                        tab.title = title.clone(); // set the title here once
                        if let Some(items) = collisions.get_mut(&title) {
                            items.push((path.clone(), ctrl_id, id));
                        } else {
                            collisions.insert(title, vec![(path.clone(), ctrl_id, id)]);
                        }
                    }
                }
            }
        }

        // walk through hashmap and update collisions with new title
        for (_, values) in &mut collisions {
            if values.len() > 1 {
                let mut splits = Vec::new();
                for (path, _, _) in values.iter_mut() {
                    // we have to find the shortest unique path combo
                    let item: Vec<String> = path.split('/').map(|v| v.to_string()).collect();
                    splits.push(item);
                }
                // compare each pair
                let mut max_equal = 0;
                for i in 0..splits.len() - 1 {
                    let a = &splits[i];
                    let b = &splits[i + 1];
                    for i in 0..a.len().min(b.len()) {
                        if a[a.len() - i - 1] != b[b.len() - i - 1] {
                            if i > max_equal {
                                max_equal = i;
                            }
                            break;
                        }
                    }
                }
                if max_equal == 0 {
                    continue;
                }
                for (index, (_, scan_ctrl_id, tab_id)) in values.iter_mut().enumerate() {
                    let split = &splits[index];
                    let mut dock_walker = self.dock.walker(dock_items);
                    while let Some((ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
                        if ctrl_id != *scan_ctrl_id {
                            continue;
                        }
                        if let DockItem::TabControl { tabs, .. } = dock_item {
                            let tab = &mut tabs[*tab_id];
                            // ok lets set the tab title
                            let new_title = if max_equal == 0 {
                                split[split.len() - 1].clone()
                            } else {
                                split[(split.len() - max_equal - 1)..].join("/")
                            };
                            if new_title != tab.title {
                                tab.title = new_title;
                            }
                        }
                    }
                }
            }
        }
        cx.request_draw();
    }

    fn new_file_editor_tab(
        &mut self,
        cx: &mut Cx,
        path: &str,
        set_last_cursor: Option<(usize, usize)>,
        at_top: bool,
        focus: bool,
    ) -> DockTab<Panel> {
        let editor_id = self.file_editors.highest_file_editor_id() + 1;
        let (file_editor, is_new) = self.file_editors.get_file_editor_for_path(path, editor_id);
        if is_new && focus {
            file_editor.set_key_focus(cx);
        }
        if let Some(cursor) = set_last_cursor {
            file_editor.set_last_cursor(cx, cursor, at_top);
        }
        DockTab {
            closeable: true,
            title: path_file_name(path),
            item: Panel::FileEditor { path: path.to_string(), scroll_pos: Vec2::default(), editor_id },
        }
    }

    fn show_log_tab(&mut self, cx: &mut Cx, _window_index: usize) {
        let mut dock_walker = self.dock.walker(&mut self.dock_items);
        while let Some((_ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
            if let DockItem::TabControl { current, tabs, .. } = dock_item {
                for (id, tab) in tabs.iter().enumerate() {
                    if let Panel::LogList = &tab.item {
                        if *current != id {
                            *current = id;
                            cx.request_draw();
                        }
                    }
                }
            }
        }
    }

    fn show_files_tab(&mut self, cx: &mut Cx, _window_index: usize) {
        let mut dock_walker = self.dock.walker(&mut self.dock_items);
        while let Some((_ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
            if let DockItem::TabControl { current, tabs, .. } = dock_item {
                for (id, tab) in tabs.iter().enumerate() {
                    if let Panel::FileTree = &tab.item {
                        if *current != id {
                            *current = id;
                            cx.request_draw();
                        }
                    }
                }
            }
        }
    }

    fn show_search_tab(&mut self, cx: &mut Cx, _window_index: usize) {
        let mut dock_walker = self.dock.walker(&mut self.dock_items);
        while let Some((_ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
            if let DockItem::TabControl { current, tabs, .. } = dock_item {
                for (id, tab) in tabs.iter().enumerate() {
                    if let Panel::SearchResults = &tab.item {
                        if *current != id {
                            *current = id;
                            cx.request_draw();
                        }
                    }
                }
            }
        }
    }

    fn show_item_display_tab(&mut self, cx: &mut Cx, _window_index: usize) {
        let mut dock_walker = self.dock.walker(&mut self.dock_items);
        while let Some((_ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
            if let DockItem::TabControl { current, tabs, .. } = dock_item {
                for (id, tab) in tabs.iter().enumerate() {
                    if let Panel::ItemDisplay = &tab.item {
                        if *current != id {
                            *current = id;
                            cx.request_draw();
                        }
                    }
                }
            }
        }
    }

    fn focus_or_new_editor(
        &mut self,
        cx: &mut Cx,
        _window_index: usize,
        file_path: &str,
        set_last_cursor: Option<(usize, usize)>,
    ) -> bool {
        let mut target_ctrl_id = None;
        let mut dock_walker = self.dock.walker(&mut self.dock_items);
        while let Some((ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
            if let DockItem::TabControl { current, tabs, .. } = dock_item {
                let mut item_ctrl_id = None;
                for (id, tab) in tabs.iter().enumerate() {
                    match &tab.item {
                        Panel::ItemDisplay => {
                            // found the editor target
                            item_ctrl_id = Some((ctrl_id, id));
                        }
                        Panel::FileEditor { path, scroll_pos: _, editor_id } => {
                            if *path == file_path {
                                // check if we aren't the preview..
                                if let Some((item_ctrl_id, tab_id)) = item_ctrl_id {
                                    if item_ctrl_id == ctrl_id && tab_id == id - 1 {
                                        continue;
                                    }
                                }
                                let (file_editor, _is_new) = self.file_editors.get_file_editor_for_path(path, *editor_id);
                                file_editor.set_key_focus(cx);
                                if let Some(cursor) = set_last_cursor {
                                    file_editor.set_last_cursor(cx, cursor, false);
                                }
                                if *current != id {
                                    *current = id;
                                    cx.request_draw();
                                }
                                return false;
                            }
                        }
                        Panel::FileEditorTarget => {
                            // found the editor target
                            target_ctrl_id = Some(ctrl_id);
                        }
                        _ => (),
                    }
                }
            }
        }
        if let Some(target_ctrl_id) = target_ctrl_id {
            // open a new one
            let new_tab = self.new_file_editor_tab(cx, file_path, set_last_cursor, false, true);
            let mut dock_walker = self.dock.walker(&mut self.dock_items);
            while let Some((ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
                if ctrl_id == target_ctrl_id {
                    if let DockItem::TabControl { current, tabs, .. } = dock_item {
                        tabs.insert(*current + 1, new_tab);
                        *current += 1;
                        cx.request_draw();
                        return true;
                    }
                }
            }
        }
        false
    }

    fn open_preview_editor_tab(
        &mut self,
        cx: &mut Cx,
        _window_index: usize,
        file_path: &str,
        set_last_cursor: Option<(usize, usize)>,
    ) -> bool {
        let mut target_ctrl_id = None;
        let mut target_tab_after = 0;
        let mut dock_walker = self.dock.walker(&mut self.dock_items);
        while let Some((ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
            if let DockItem::TabControl { tabs, .. } = dock_item {
                for (id, tab) in tabs.iter().enumerate() {
                    match &tab.item {
                        Panel::ItemDisplay => {
                            // found the editor target
                            target_ctrl_id = Some(ctrl_id);
                            target_tab_after = id;
                        }
                        _ => (),
                    }
                }
            }
        }
        if let Some(target_ctrl_id) = target_ctrl_id {
            // open a new one
            let mut dock_walker = self.dock.walker(&mut self.dock_items);
            while let Some((ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
                if ctrl_id == target_ctrl_id {
                    if let DockItem::TabControl { current, tabs, .. } = dock_item {
                        // already contains the editor we need, or if we need a new one
                        // check what tab is right next to ItemDisplay
                        if target_tab_after + 1 < tabs.len() {
                            match &mut tabs[target_tab_after + 1].item {
                                Panel::FileEditor { path, scroll_pos: _, editor_id } => {
                                    if self.file_editors.does_path_match_editor_type(file_path, *editor_id) {
                                        *path = file_path.to_string();
                                        let (file_editor, _is_new) = self.file_editors.get_file_editor_for_path(path, *editor_id);
                                        if let Some(cursor) = set_last_cursor {
                                            file_editor.set_last_cursor(cx, cursor, true);
                                        }
                                        *current = target_tab_after + 1;
                                        cx.request_draw();
                                        return true;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
        }

        if let Some(target_ctrl_id) = target_ctrl_id {
            // open a new one
            let new_tab = self.new_file_editor_tab(cx, file_path, set_last_cursor, true, false);
            let mut dock_walker = self.dock.walker(&mut self.dock_items);
            while let Some((ctrl_id, dock_item)) = dock_walker.walk_dock_item() {
                if ctrl_id == target_ctrl_id {
                    if let DockItem::TabControl { current, tabs, .. } = dock_item {
                        tabs.insert(target_tab_after + 1, new_tab);
                        *current = target_tab_after + 1;
                        cx.request_draw();
                        return true;
                    }
                }
            }
        }

        false
    }
}
