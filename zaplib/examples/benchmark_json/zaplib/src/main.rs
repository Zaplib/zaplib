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

fn calc(s: &str) -> Coordinate {
    let jobj = serde_json::from_str::<TestStruct>(s).unwrap();

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

fn run() {
    let mut file = UniversalFile::open("zaplib/examples/benchmark_json/data.json").unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s);

    let start = Instant::now();
    calc(&s);
    let end: UniversalInstant = Instant::now();
    log!("{:?}", end.duration_since(start));
}

fn call_rust(_name: String, _params: Vec<ZapParam>) -> Vec<ZapParam> {
    run();

    vec![]
}

register_call_rust!(call_rust);
