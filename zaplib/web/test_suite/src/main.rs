use std::sync::Arc;

use zaplib::*;
use zaplib_components::*;

const PANIC_STATUS: StatusId = location_hash!();

pub struct TestSuiteApp {
    window: Window,
    pass: Pass,
    main_view: View,
    send_button: Button,
    worker_button: Button,
    dump_button: Button,
    buffers: Vec<Arc<Vec<u8>>>,
    signal: Signal,
    panic_draw: bool,
}

impl TestSuiteApp {
    pub fn new(cx: &mut Cx) -> Self {
        cx.on_call_rust_in_same_thread_sync(Self::call_rust_in_same_thread_sync);
        cx.on_call_rust(Self::on_call_rust);
        let buffer = Arc::new(vec![1; 8]);
        let buffers = vec![buffer];
        Self {
            window: Window {
                create_inner_size: Some(vec2(1000., 700.)),
                #[cfg(not(target_arch = "wasm32"))]
                create_cef_url: Some("http://localhost:3000/zaplib/web/test_suite/index.html".to_string()),
                ..Window::default()
            },
            pass: Pass::default(),
            main_view: View::default(),
            send_button: Button::default(),
            worker_button: Button::default(),
            dump_button: Button::default(),
            signal: cx.new_signal(),
            buffers,
            panic_draw: false,
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        match event {
            Event::Signal(sig) => {
                if let Some(statusses) = sig.signals.get(&self.signal) {
                    for status in statusses {
                        if *status == PANIC_STATUS {
                            panic!("Panic signal");
                        }
                    }
                } else {
                    // call_rust_in_same_thread_sync send_signal uses a fake signal ID
                    log!("received signal!");
                }
            }
            _ => {}
        }

        if let ButtonEvent::Clicked = self.send_button.handle(cx, event) {
            let mut params = vec!["hello world :-)".to_string().into_param()];
            for buffer in &self.buffers {
                params.push(buffer.clone().into_param());
            }
            cx.call_js("log", params);
        }
        if let ButtonEvent::Clicked = self.worker_button.handle(cx, event) {
            let buffer = Arc::new(vec![1; 8]);
            let params = vec![Arc::clone(&buffer).into_param()];
            cx.call_js("sendWorker", params);
            self.buffers.push(buffer);
        }
        if let ButtonEvent::Clicked = self.dump_button.handle(cx, event) {
            log!("Total buffers: {}", self.buffers.len());
            for buffer in self.buffers.iter() {
                let count = Arc::strong_count(buffer);
                if count > 1 {
                    log!("Buffer arc={} ptr={}, rc = {}", Arc::as_ptr(buffer) as u32, buffer.as_ptr() as u32, count);
                }
            }
        }
    }

    fn on_call_rust(&mut self, cx: &mut Cx, name: String, params: Vec<ZapParam>) -> Vec<ZapParam> {
        match name.as_str() {
            "array_multiply_u8" => {
                let value: u8 = serde_json::from_str(params[0].as_str()).unwrap();
                let b = params[1].as_u8_slice();
                vec![Arc::<Vec<u8>>::new(b.iter().map(|x| *x * value).collect()).into_param()]
            }
            "array_multiply_u8_readonly" => {
                let value: u8 = serde_json::from_str(params[0].as_str()).unwrap();
                let b = params[1].as_u8_slice();
                vec![Arc::<Vec<u8>>::new(b.iter().map(|x| *x * value).collect()).into_param()]
            }
            "array_multiply_f32" => {
                let value: f32 = serde_json::from_str(params[0].as_str()).unwrap();
                let b = params[1].as_f32_slice();
                vec![Arc::<Vec<f32>>::new(b.iter().map(|x| *x * value).collect()).into_param()]
            }
            "array_multiply_f32_readonly" => {
                let value: f32 = serde_json::from_str(params[0].as_str()).unwrap();
                let b = params[1].as_f32_slice();
                vec![Arc::<Vec<f32>>::new(b.iter().map(|x| *x * value).collect()).into_param()]
            }
            "total_sum" => {
                let buffer = params[0].as_u8_slice();
                let sum: u8 = buffer.iter().sum();
                vec![sum.to_string().into_param()]
            }
            "call_rust_no_return" => {
                // Note: not returning anything to test destructor behavior
                vec![]
            }
            "check_arc_count" => {
                let arc_ptr = params[0].as_str().parse::<u64>().unwrap() as *const Vec<u8>;
                let arc: Arc<Vec<u8>> = unsafe { Arc::from_raw(arc_ptr) };
                let count = Arc::strong_count(&arc);
                let _ = Arc::into_raw(arc);
                vec![vec![count as u8].into_param()]
            }
            "panic" => {
                panic!("I am panicking!");
            }
            "panic_signal" => {
                Cx::post_signal(self.signal, PANIC_STATUS);
                vec![]
            }
            "panic_draw" => {
                self.panic_draw = true;
                cx.request_draw();
                vec![]
            }
            unknown_name => {
                panic!("Unknown function name: {}", unknown_name)
            }
        }
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        if self.panic_draw {
            panic!("Panic draw");
        }

        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::all(0.));

        self.main_view.begin_view(cx, LayoutSize::FILL);

        cx.add_box(LayoutSize::new(Width::Fix(0.), Height::Fix(30.)));
        cx.begin_right_box();
        self.send_button.draw(cx, "send log event");
        self.worker_button.draw(cx, "send to worker");
        self.dump_button.draw(cx, "dump rc counts");
        cx.end_right_box();

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }

    pub fn call_rust_in_same_thread_sync(name: &str, params: Vec<ZapParam>) -> Vec<ZapParam> {
        match name {
            "array_multiply_u8" => {
                let value: u8 = serde_json::from_str(params[0].as_str()).unwrap();
                let b = params[1].as_u8_slice();
                vec![Arc::<Vec<u8>>::new(b.iter().map(|x| *x * value).collect()).into_param()]
            }
            "array_multiply_f32" => {
                let value: f32 = serde_json::from_str(params[0].as_str()).unwrap();
                let b = params[1].as_f32_slice();
                vec![Arc::<Vec<f32>>::new(b.iter().map(|x| *x * value).collect()).into_param()]
            }
            "array_multiply_f32_readonly" => {
                let value: f32 = serde_json::from_str(params[0].as_str()).unwrap();
                let b = params[1].as_f32_slice();
                vec![Arc::<Vec<f32>>::new(b.iter().map(|x| *x * value).collect()).into_param()]
            }
            "send_signal" => {
                // This is a fake signal ID
                Cx::post_signal(Signal { signal_id: 123 }, location_hash!());
                Vec::new()
            }
            unknown => {
                panic!("Unknown function name: {}", unknown);
            }
        }
    }
}

main_app!(TestSuiteApp);
