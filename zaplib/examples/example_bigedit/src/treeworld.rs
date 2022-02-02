// a bunch o buttons to select the world
use zaplib::*;
use zaplib_components::*;

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        code_fragment!(
            r#"
            uniform time: float;
            uniform max_depth: float;

            const basecolor: vec4 = #E27D3A;
            const leaf_1: vec4 = #C1FF00;
            const leaf_2: vec4 = #009713;
            const angle: float = 0.500;
            const off: float = 0.183;
            const width: float = 0.3;
            const alpha: float = 0.114;

            geometry geom: vec2;

            instance in_path: float;
            instance depth: float;

            varying color: vec4;
            fn vertex() -> vec4 {
                let pos = vec2(0.0, 0.5);
                let scale = vec2(0.2, 0.2);
                let dir = vec2(0.0, 0.8);
                let smaller = vec2(.85, 0.85);
                let path = in_path;
                let nodesize = vec2(1.);
                let z = 0.0;
                let last_z = 0.0;
                let z_base = -1.5;
                for i from 0 to 20 {
                    if float(i) >= depth {
                        break;
                    }

                    let turn_right = mod (path, 2.);
                    let turn_fwd = mod (path, 8.);
                    let angle = 50.*angle;
                    last_z = z;
                    if (turn_right > 0.) {
                        angle = -1.0 * angle;
                    }
                    if(turn_fwd > 3.){
                        z += 0.4 * scale.x;
                    }
                    else{
                        z -= 0.4 * scale.x;
                    }
                    z += sin(time + 10. * pos.x)*0.01;
                    angle += sin(time + 10. * pos.x) * 5.;

                    let d_left = max(0.1 - length(-vec3(pos, z_base + z)), 0.) * 300.0;
                    let d_right = max(0.1 - length(-vec3(pos, z_base + z)), 0.) * 300.0;

                    angle -= d_left;
                    angle += d_right;

                    dir = Math::rotate_2d(dir, angle * TORAD);
                    pos += dir * scale;
                    scale = scale * smaller;
                    path = floor(path / 2.);
                }
                let size = vec2(0.01, 0.01);

                let m = Math::rotate_2d(
                    vec2(1.0, width) * (geom.xy * nodesize - vec2(5.0*off, 0.5)),
                    atan(
                        dir.y,
                        dir.x
                    )
                );

                let v = vec4(
                    m * scale.xy + pos.xy,
                    z_base + mix(last_z, z, geom.y),
                    1.
                );

                return camera_projection * (camera_view * v);
            }

            fn pixel() -> vec4 {
                let color = vec4(0.);
                if depth > max_depth{
                    color = mix(leaf_1,leaf_2,sin(0.01*in_path));
                }
                else{
                    color = basecolor;
                }
                return vec4(color.xyz * alpha, alpha);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub struct TreeWorld {
    pub area: Area,
}

#[repr(C)]
struct TreeWorldUniforms {
    time: f32,
    max_depth: f32,
}

const MAX_DEPTH: f32 = 10.0;

/*
Low    quest1
Medium quest2
High   pcbase
Ultra  pchigh
*/
impl TreeWorld {
    fn animate(&self, cx: &mut Cx) {
        self.area.write_user_uniforms(cx, TreeWorldUniforms { time: cx.last_event_time as f32, max_depth: MAX_DEPTH });
        cx.request_next_frame();
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let Event::NextFrame = event {
            self.animate(cx);
        }
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        SkyBox::draw(cx, vec3(0., 0., 0.));

        let mut instances = vec![];
        fn recur(instances: &mut Vec<(f32, f32)>, path: f32, depth: f32, max_depth: f32) {
            instances.push((path, depth));
            if depth > max_depth {
                return;
            }
            recur(instances, path, depth + 1.0, max_depth);
            recur(instances, path + (2.0f32).powf(depth), depth + 1.0, max_depth);
        }
        recur(&mut instances, 0., 0., MAX_DEPTH);

        self.area = cx.add_instances(&SHADER, &instances);

        self.animate(cx);
    }
}
