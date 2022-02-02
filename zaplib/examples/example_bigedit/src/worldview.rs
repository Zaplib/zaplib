use std::f32::consts::PI;

// a bunch o buttons to select the world
use crate::fieldworld::FieldWorld;
use crate::treeworld::TreeWorld;
use zaplib::*;
use zaplib_components::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Ord)]
enum WorldType {
    TreeWorld,
    FieldWorld,
}

impl WorldType {
    fn name(&self) -> String {
        match self {
            Self::TreeWorld => "TreeWorld".to_string(),
            Self::FieldWorld => "FieldWorld".to_string(),
        }
    }
}

const WORLD_TYPES: &[WorldType] = &[WorldType::TreeWorld, WorldType::FieldWorld];

pub struct WorldView {
    select_view: ScrollView,
    buttons: Vec<Button>,
    view: View,
    bg: Background,
    xr_is_presenting: bool,
    viewport_3d: Viewport3D,
    world_type: WorldType,
    tree_world: TreeWorld,
    field_world: FieldWorld,
}

const COLOR_BG: Vec4 = vec4(34.0 / 255.0, 34.0 / 255.0, 34.0 / 255.0, 1.0);

const VIEWPORT_PROPS: Viewport3DProps = Viewport3DProps {
    camera_target: Vec3 { x: 0.0, y: 0.5, z: -1.5 },
    initial_camera_position: Coordinates::Spherical(SphericalAngles { phi: PI / 2., theta: 0., radius: 1.5 + 1.1 }),
    panning_enabled: false,
    ..Viewport3DProps::DEFAULT
};

impl Default for WorldView {
    fn default() -> Self {
        Self {
            view: View::default(),
            bg: Background::default(),
            select_view: ScrollView::new_standard_vh(),
            viewport_3d: Viewport3D::default(),
            buttons: WORLD_TYPES.iter().map(|_| Button::default()).collect(),
            world_type: WorldType::TreeWorld,
            xr_is_presenting: false,
            tree_world: TreeWorld::default(),
            field_world: FieldWorld::default(),
        }
    }
}

impl WorldView {
    pub fn handle_world_select(&mut self, cx: &mut Cx, event: &mut Event) {
        if self.select_view.handle(cx, event) {}
        for (index, btn) in self.buttons.iter_mut().enumerate() {
            if let ButtonEvent::Clicked = btn.handle(cx, event) {
                self.world_type = WORLD_TYPES[index];
                cx.request_draw();
            }
        }
    }

    pub fn draw_world_select(&mut self, cx: &mut Cx) {
        self.select_view.begin_view(cx, LayoutSize::FILL);
        self.bg.begin_draw(cx, Width::Fill, Height::Fill, COLOR_BG);

        for (index, button) in self.buttons.iter_mut().enumerate() {
            button.draw(cx, &WORLD_TYPES[index].name());
        }

        self.bg.end_draw(cx);
        self.select_view.end_view(cx);
    }

    pub fn handle_world_view(&mut self, cx: &mut Cx, event: &mut Event) {
        // do 2D camera interaction.
        if !self.xr_is_presenting {
            self.viewport_3d.handle(cx, event);
        }

        match &self.world_type {
            WorldType::TreeWorld => {
                self.tree_world.handle(cx, event);
            }
            WorldType::FieldWorld => {
                self.field_world.handle(cx, event);
            }
        }
    }

    pub fn draw_world_view_2d(&mut self, cx: &mut Cx) {
        self.viewport_3d.begin_draw(cx, VIEWPORT_PROPS);
        self.draw_world_view_3d(cx);
        self.viewport_3d.end_draw(cx);
    }

    pub fn draw_world_view_3d(&mut self, cx: &mut Cx) {
        cx.begin_absolute_box();
        self.view.begin_view(cx, LayoutSize::FILL);

        match &self.world_type {
            WorldType::TreeWorld => {
                self.tree_world.draw(cx);
            }
            WorldType::FieldWorld => {
                self.field_world.draw(cx);
            }
        }

        self.view.end_view(cx);
        cx.end_absolute_box();
    }
}
