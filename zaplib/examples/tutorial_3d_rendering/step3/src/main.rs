use std::io::Read;

use zaplib::{
    byte_extract::{get_f32_le, get_u32_le},
    *,
};
use zaplib_components::*;

#[derive(Clone, Copy)]
#[repr(C)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Instance {
    offset: f32,
    color: Vec3,
}

const INSTANCES: [Instance; 3] = [
    Instance { offset: -10., color: vec3(1., 1., 0.) },
    Instance { offset: 0., color: vec3(0., 1., 1.) },
    Instance { offset: 10., color: vec3(1., 0., 1.) },
];

static SHADER: Shader = Shader {
    build_geom: None,
    code_to_concatenate: &[
        Cx::STD_SHADER,
        code_fragment!(
            r#"
            instance offset: float;
            instance color: vec3;

            geometry position: vec3;
            geometry normal: vec3;

            fn vertex() -> vec4 {
                return camera_projection * camera_view * vec4(vec3(position.x, position.y + offset, position.z), 1.);
            }

            fn pixel() -> vec4 {
                let lightPosition = vec3(20.,0.,30.);
                let lightDirection = normalize(position - lightPosition);
                return vec4(clamp(dot(-lightDirection, normal), 0.0, 1.0) * color,1.0);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
struct App {
    window: Window,
    pass: Pass,
    view: View,
    viewport_3d: Viewport3D,
    geometry: Option<GpuGeometry>,
}

fn parse_stl(cx: &mut Cx, url: &str) -> GpuGeometry {
    let mut file = UniversalFile::open(url).unwrap();
    let mut data = vec![];
    file.read_to_end(&mut data).unwrap();

    const HEADER_LENGTH: usize = 80;
    let num_triangles = get_u32_le(&data, HEADER_LENGTH) as usize;

    let vertices: Vec<Vertex> = (0..num_triangles)
        .flat_map(|i| {
            let offset: usize = HEADER_LENGTH + 4 + i * 50;
            let normal = vec3(get_f32_le(&data, offset), get_f32_le(&data, offset + 4), get_f32_le(&data, offset + 8));

            [
                Vertex {
                    position: vec3(
                        get_f32_le(&data, offset + 12),
                        get_f32_le(&data, offset + 16),
                        get_f32_le(&data, offset + 20),
                    ),
                    normal,
                },
                Vertex {
                    position: vec3(
                        get_f32_le(&data, offset + 24),
                        get_f32_le(&data, offset + 28),
                        get_f32_le(&data, offset + 32),
                    ),
                    normal,
                },
                Vertex {
                    position: vec3(
                        get_f32_le(&data, offset + 36),
                        get_f32_le(&data, offset + 40),
                        get_f32_le(&data, offset + 44),
                    ),
                    normal,
                },
            ]
        })
        .collect();

    let indices = (0..num_triangles as u32).map(|i| [i * 3, i * 3 + 1, i * 3 + 2]).collect();
    GpuGeometry::new(cx, Geometry::new(vertices, indices))
}

impl App {
    fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        self.viewport_3d.handle(cx, event);

        if let Event::Construct = event {
            self.geometry = Some(parse_stl(cx, "zaplib/examples/tutorial_3d_rendering/teapot.stl"));
            cx.request_draw();
        }
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("300"));
        self.view.begin_view(cx, LayoutSize::FILL);
        if let Some(geometry) = &self.geometry {
            self.viewport_3d.begin_draw(
                cx,
                Viewport3DProps {
                    initial_camera_position: Coordinates::Cartesian(vec3(0., -30., 30.)),
                    ..Viewport3DProps::DEFAULT
                },
            );
            cx.add_mesh_instances(&SHADER, &INSTANCES, geometry.clone());
            self.viewport_3d.end_draw(cx);
        }
        self.view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(App);
