use zaplib::*;

fn sum(values: &[u8]) -> u8 {
    values.iter().sum()
}

fn call_rust(name: String, params: Vec<ZapParam>) -> Vec<ZapParam> {
    if name == "sum" {
        let values = params[0].as_u8_slice();
        let response = vec![sum(values)].into_param();
        return vec![response];
    }

    vec![]
}

register_call_rust!(call_rust);
