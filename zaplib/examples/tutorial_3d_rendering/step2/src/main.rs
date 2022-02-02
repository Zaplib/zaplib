use std::io::Read;
use zaplib::{
    byte_extract::{get_f32_le, get_u32_le},
    *,
};

fn parse_stl() -> Vec<ZapParam> {
    let mut file = UniversalFile::open("zaplib/examples/tutorial_3d_rendering/teapot.stl").unwrap();
    let mut data = vec![];
    file.read_to_end(&mut data).unwrap();

    const HEADER_LENGTH: usize = 80;
    let num_triangles = get_u32_le(&data, HEADER_LENGTH) as usize;
    let mut vertices = Vec::with_capacity(num_triangles * 9);
    let mut normals = Vec::with_capacity(num_triangles * 9);
    for i in 0..num_triangles {
        let offset = HEADER_LENGTH + 4 + i * 50;

        let normal_x = get_f32_le(&data, offset);
        let normal_y = get_f32_le(&data, offset + 4);
        let normal_z = get_f32_le(&data, offset + 8);

        for j in (0..36).step_by(12) {
            vertices.push(get_f32_le(&data, offset + 12 + j));
            vertices.push(get_f32_le(&data, offset + 16 + j));
            vertices.push(get_f32_le(&data, offset + 20 + j));

            normals.push(normal_x);
            normals.push(normal_y);
            normals.push(normal_z);
        }
    }

    vec![vertices.into_param(), normals.into_param()]
}

fn call_rust(name: String, _params: Vec<ZapParam>) -> Vec<ZapParam> {
    if name == "parse_stl" {
        parse_stl()
    } else {
        panic!("Unknown function name");
    }
}

register_call_rust!(call_rust);
