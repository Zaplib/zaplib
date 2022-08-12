// Based on https://github.com/kostya/benchmarks/blob/1dd7deb29a813d1095e6062c25ad92bd81ce0273/json/json.rs/src/json_struct.rs

use serde::Deserialize;
use std::io::Read;
use zaplib::*;

#[derive(Deserialize, PartialEq)]
struct Coordinate {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Deserialize)]
struct TestStruct {
    coordinates: Vec<Coordinate>,
}

fn calc(jobj: TestStruct) -> Coordinate {
    let len = jobj.coordinates.len() as f64;
    let mut x = 0_f64;
    let mut y = 0_f64;
    let mut z = 0_f64;

    for coord in &jobj.coordinates {
        x += coord.x;
        y += coord.y;
        z += coord.z;
    }

    Coordinate { x: x / len, y: y / len, z: z / len }
}

fn call_rust(_name: String, _params: Vec<ZapParam>) -> Vec<ZapParam> {
    let mut file = UniversalFile::open("zaplib/examples/benchmark_json/data.json").unwrap();
    let mut s = String::new();
    let ret = file.read_to_string(&mut s);
    if ret.is_err() {
        panic!("Failed to read file");
    }

    let start_p = Instant::now();
    let jobj = serde_json::from_str::<TestStruct>(&s).unwrap();
    let end_p: UniversalInstant = Instant::now();

    let start = Instant::now();
    calc(jobj);
    let end: UniversalInstant = Instant::now();

    vec![vec![end_p.duration_since(start_p).as_millis() as u32, end.duration_since(start).as_millis() as u32].into_param()]
}

register_call_rust!(call_rust);
