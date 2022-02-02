use std::f32::consts::PI;

use zaplib::*;

/// Carefully chosen so that at the poles (all the way up or down) you can still rotate
/// nicely.
const EPSILON: f32 = 0.0001;

/// A nice article about how a 3D camera's look_at function works:
/// <https://www.scratchapixel.com/lessons/mathematics-physics-for-computer-graphics/lookat-function>
fn look_at(eye: Vec3, at: Vec3, up: Vec3) -> Mat4 {
    let forward = (eye - at).normalize();
    let left = Vec3::cross(up, forward).normalize();
    let up = Vec3::cross(forward, left);

    let mut matrix = Mat4::identity();
    matrix.v[0] = left.x;
    matrix.v[4] = left.y;
    matrix.v[8] = left.z;
    matrix.v[1] = up.x;
    matrix.v[5] = up.y;
    matrix.v[9] = up.z;
    matrix.v[2] = forward.x;
    matrix.v[6] = forward.y;
    matrix.v[10] = forward.z;
    matrix.v[12] = -left.dot(eye);
    matrix.v[13] = -up.dot(eye);
    matrix.v[14] = -forward.dot(eye);
    matrix
}

/// Spherical coordinates follow the same conventions as <https://threejs.org/docs/#api/en/math/Spherical>
#[derive(Clone, Copy, Debug)]
pub struct SphericalAngles {
    /// Polar angle from 0 to PI. A value of 0 looking down the Y axis, and PI looking up the Y axis.
    pub phi: f32,
    /// Equator angle around the Y (up) axis from 0 to 2*PI. Example values:
    ///   - 0 looking down the Z axis
    ///   - PI/2 looking down the X axis
    ///   - PI looking up the Z axis
    ///   - 3*PI/2 looking up the X axis.
    pub theta: f32,
    // Distance from camera target
    pub radius: f32,
}

pub enum Coordinates {
    Cartesian(Vec3),
    Spherical(SphericalAngles),
}

fn cartesian_to_spherical(position: Vec3) -> SphericalAngles {
    let radius = position.length();
    SphericalAngles { phi: position.z.atan2(position.y), theta: (position.x / radius).asin(), radius }
}

pub struct Viewport3DProps {
    pub initial_camera_position: Coordinates,
    /// Represents if users can use the left mouse to pan the camera.
    pub panning_enabled: bool,
    pub camera_target: Vec3,
    /// Represents if panning should move camera vertically.
    pub vertical_panning_enabled: bool,
}

impl Viewport3DProps {
    /// TODO(JP): Replace these with TextInsProps::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Self = Self {
        initial_camera_position: Coordinates::Spherical(SphericalAngles { phi: EPSILON, theta: -PI / 2., radius: 50. }),
        camera_target: Vec3::all(0.),
        panning_enabled: true,
        vertical_panning_enabled: true,
    };
}

impl Default for Viewport3DProps {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub struct Viewport3D {
    component_id: ComponentId,
    area: Area,
    pass: Pass,
    clear_color: Vec4,
    pub color_texture: Texture,
    depth_texture: Texture,
    view_2d: View,
    view_3d: View,
    pub measured_size: Vec2,
    camera_position: SphericalAngles,
    camera_position_start: Option<SphericalAngles>,
    camera_target_offset: Vec3,
    camera_target_offset_start: Option<Vec3>,
    props: Viewport3DProps,
    has_read_props: bool,
}

impl Default for Viewport3D {
    fn default() -> Self {
        Self {
            component_id: Default::default(),
            area: Default::default(),
            // Make_safe concept borrowed from ThreeJS:
            // https://github.com/mrdoob/three.js/blob/342946c8392639028da439b6dc0597e58209c696/src/math/Spherical.js#L43
            camera_position: SphericalAngles { phi: EPSILON, theta: -PI / 2., radius: 50. },
            measured_size: Default::default(),
            camera_position_start: Default::default(),
            camera_target_offset: Default::default(),
            camera_target_offset_start: Default::default(),
            pass: Default::default(),
            clear_color: Default::default(),
            color_texture: Default::default(),
            depth_texture: Default::default(),
            view_3d: Default::default(),
            view_2d: Default::default(),
            has_read_props: Default::default(),
            props: Default::default(),
        }
    }
}

impl Viewport3D {
    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> Option<PassMatrixMode> {
        match event.hits_pointer(cx, self.component_id, self.area.get_rect_for_first_instance(cx)) {
            Event::PointerHover(_pe) => {
                // cx.set_hover_mouse_cursor(MouseCursor::Move);
            }
            // traditional mouse down
            Event::PointerDown(pe) => {
                // cx.set_down_mouse_cursor(MouseCursor::Move);
                if self.props.panning_enabled && pe.button == MouseButton::Left {
                    self.camera_target_offset_start = Some(self.camera_target_offset);
                } else if pe.button == MouseButton::Right {
                    self.camera_position_start = Some(self.camera_position);
                }
            }
            // traditional mouse up
            Event::PointerUp(_pe) => {
                self.camera_position_start = None;
                self.camera_target_offset_start = None;
            }
            Event::PointerScroll(pe) => {
                let min_distance = 1.0; // a little more than near
                let max_distance = 900.; // a little less than far
                let zoom_speed = (self.camera_position.radius * (PI / 4.) / max_distance).sin().abs() / 2.0;
                self.camera_position.radius =
                    (self.camera_position.radius + pe.scroll.y * zoom_speed).max(min_distance).min(max_distance);
                return Some(self.pass_set_matrix_mode(cx));
            }
            Event::PointerMove(pe) => {
                // Using standard makeSafe approach to clamp to slightly less than the limits for phi/theta
                // Concept borrowed from ThreeJS:
                // https://github.com/mrdoob/three.js/blob/342946c8392639028da439b6dc0597e58209c696/src/math/Spherical.js#L43
                if let Some(SphericalAngles { phi, theta, radius }) = self.camera_position_start {
                    let rotate_speed = 1. / 175.;
                    self.camera_position = SphericalAngles {
                        theta: (theta - (pe.abs.x - pe.abs_start.x) * rotate_speed) % (PI * 2.),
                        phi: (phi - (pe.abs.y - pe.abs_start.y) * rotate_speed).clamp(EPSILON, PI - EPSILON),
                        radius,
                    };
                    return Some(self.pass_set_matrix_mode(cx));
                } else if let Some(camera_target_offset_start) = self.camera_target_offset_start {
                    // TODO(Shobhit): Whenever we do Orthographic view properly, we need to adjust the panning accordingly
                    // We would need to consider viewable area's width and height into consideration just like how
                    // worldview does it:
                    // https://git.io/J0wsP
                    // Please refer some more discussion about this here:
                    // https://github.robot.car/cruise/exviz/pull/107#discussion_r932946
                    let pan_speed = 0.8;
                    // Normalize using the height of the viewport and the camera distance, since those determine the field of view
                    // intersecting with the camera target.
                    let mouse_offset = (pe.rel - pe.rel_start) / self.measured_size.y * self.camera_position.radius * pan_speed;

                    let vertical_offset =
                        if self.props.vertical_panning_enabled { self.camera_position.phi.to_degrees() } else { 0. };

                    // We need to calculate the value of camera target offset.
                    // For that we create a rotation matrix from the camera_position (the rotation),
                    // thereafter we translate it on x/y based on relative offset calculated mouse movements.
                    // Finally, so that we don't forget about the previous camera target offsets,
                    // we add camera_target_offset_start so that we don't start from beginning in every interaction.
                    self.camera_target_offset = Mat4::rotation(vertical_offset, self.camera_position.theta.to_degrees(), 0.)
                        .transform_vec4(vec4(-mouse_offset.x, 0., -mouse_offset.y, 1.0))
                        .to_vec3()
                        + camera_target_offset_start;

                    return Some(self.pass_set_matrix_mode(cx));
                }
            }
            _ => (),
        }

        None
    }

    fn get_matrix_projection(&self) -> PassMatrixMode {
        let SphericalAngles { phi, theta, radius } = self.camera_position;
        let eye = radius * vec3(phi.sin() * theta.sin(), phi.cos(), phi.sin() * theta.cos());

        PassMatrixMode::Projection {
            fov_y: 40.0,
            near: 0.1,
            far: 1000.0,
            cam: look_at(
                eye + self.props.camera_target + self.camera_target_offset,
                self.props.camera_target + self.camera_target_offset,
                vec3(0., 1., 0.),
            ),
        }
    }

    fn pass_set_matrix_mode(&mut self, cx: &mut Cx) -> PassMatrixMode {
        let matrix_mode = self.get_matrix_projection();
        self.pass.set_matrix_mode(cx, matrix_mode.clone());
        matrix_mode
    }

    /// TODO(JP): This is kind of exploiting a potential bug in the framework.. We don't clean up [`Pass`]es,
    /// so if we just don't call [`Pass::begin_pass`] then it will happily keep on rendering. Is this a bug
    /// or a feature? I'm not sure.. See [`Pass::begin_pass`] for more thoughts.
    #[must_use]
    pub fn skip_draw(&mut self, cx: &mut Cx) -> bool {
        // We have to manually check if the size has changed. See [`Pass:begin_pass`] for more info.
        if self.measured_size != vec2(cx.get_width_total(), cx.get_height_total()) {
            return false;
        }
        self.draw_viewport_2d(cx);
        true
    }

    pub fn begin_draw(&mut self, cx: &mut Cx, props: Viewport3DProps) {
        if !self.has_read_props {
            self.camera_position = match props.initial_camera_position {
                Coordinates::Cartesian(cartesian) => cartesian_to_spherical(cartesian),
                Coordinates::Spherical(spherical) => spherical,
            };
            self.has_read_props = true;
        }
        self.props = props;

        self.draw_viewport_2d(cx);

        self.pass.begin_pass_without_textures(cx);

        self.pass.set_size(cx, self.measured_size);
        let color_texture_handle = self.color_texture.get_color(cx);
        self.pass.add_color_texture(cx, color_texture_handle, ClearColor::ClearWith(self.clear_color));
        let depth_texture_handle = self.depth_texture.get_depth(cx);
        self.pass.set_depth_texture(cx, depth_texture_handle, ClearDepth::ClearWith(1.0));

        self.view_3d.begin_view(cx, LayoutSize::FILL);
    }

    pub fn end_draw(&mut self, cx: &mut Cx) -> PassMatrixMode {
        let matrix_mode = self.pass_set_matrix_mode(cx);

        self.view_3d.end_view(cx);
        self.pass.end_pass(cx);

        matrix_mode
    }

    fn draw_viewport_2d(&mut self, cx: &mut Cx) {
        self.view_2d.begin_view(cx, LayoutSize::FILL);
        // blit the texture to a view rect
        self.measured_size = vec2(cx.get_width_total(), cx.get_height_total());
        let color_texture_handle = self.color_texture.get_color(cx);
        self.area = ImageIns::draw(cx, Rect { pos: cx.get_box_origin(), size: self.measured_size }, color_texture_handle);

        self.view_2d.end_view(cx);
    }
}
