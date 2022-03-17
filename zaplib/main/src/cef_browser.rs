use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Result};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, RwLock};
use std::{env, process};

use crate::cef_utils::vec4_to_cef_color;
use crate::cx_web::parse_keyboard_event_from_js;
use crate::zerde::*;
use crate::*;
use zaplib_cef::{
    create_browser_sync, execute_process, initialize, App, Browser, BrowserProcessHandler, BrowserSettings, Client, CommandLine,
    ContextMenuHandler, Frame, MenuModel, ProcessId, ProcessMessage, RenderProcessHandler, RequestHandler, ResourceHandler,
    ResourceRequestHandler, Settings, V8ArrayBufferReleaseCallback, V8Context, V8PropertyAttribute, V8Value, WindowHandle,
    WindowInfo,
};

/// Represents a CEF browser that might not actually be initialized yet, but which will still queue up
/// any calls to [`MaybeCefBrowser::call_js`].
pub(crate) enum MaybeCefBrowser {
    #[allow(dead_code)] // We never initialize in win/linux currently.
    Initialized(CefBrowser),
    Uninitialized {
        /// Already create a channel so it will queue up the messages that we send with [`MaybeCefBrowser::call_js`].
        messages_channel: (mpsc::Sender<CallJsEvent>, mpsc::Receiver<CallJsEvent>),
        /// Save this for when initializing.
        call_rust_sync_fn: Option<CallRustSyncFn>,
    },
}

impl MaybeCefBrowser {
    pub(crate) fn new() -> Self {
        Self::Uninitialized { messages_channel: mpsc::channel(), call_rust_sync_fn: None }
    }

    /// Queues up messages if the browser isn't initialized yet; otherwise calls them directly.
    pub(crate) fn call_js(&mut self, name: &str, params: Vec<ZapParam>) {
        let call_js_event = CallJsEvent { name: name.to_string(), params };
        match self {
            MaybeCefBrowser::Initialized(cef_browser) => cef_browser.call_js(call_js_event),
            MaybeCefBrowser::Uninitialized { messages_channel, .. } => messages_channel.0.send(call_js_event).unwrap(),
        }
    }

    pub(crate) fn on_call_rust_sync(&mut self, func: CallRustSyncFn) {
        match self {
            MaybeCefBrowser::Initialized(cef_browser) => cef_browser.on_call_rust_sync(func),
            MaybeCefBrowser::Uninitialized { call_rust_sync_fn, .. } => *call_rust_sync_fn = Some(func),
        }
    }

    /// Turn the browser into an initialized browser.
    #[allow(dead_code)] // We never initialize in win/linux currently.
    pub(crate) fn initialize(
        &mut self,
        size: Vec2,
        url: &str,
        parent_window: WindowHandle,
        #[cfg(feature = "cef-server")] get_resource_url_callback: Option<GetResourceUrlCallback>,
    ) {
        match self {
            MaybeCefBrowser::Initialized(_) => {
                panic!("CEF is already initialized; we currently support only one browser at a time")
            }
            MaybeCefBrowser::Uninitialized { messages_channel, call_rust_sync_fn } => {
                let mut channel = mpsc::channel();
                std::mem::swap(&mut channel, messages_channel);
                // Pass in the existing channel so we can process our queued messages.
                let mut cef_browser = CefBrowser::new(
                    size,
                    url,
                    parent_window,
                    channel,
                    #[cfg(feature = "cef-server")]
                    get_resource_url_callback,
                );
                if let Some(func) = call_rust_sync_fn {
                    cef_browser.on_call_rust_sync(*func);
                }
                *self = MaybeCefBrowser::Initialized(cef_browser);
            }
        }
    }

    #[allow(dead_code)] // We never call this in win/linux currently.
    pub(crate) fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        match self {
            MaybeCefBrowser::Initialized(cef_browser) => cef_browser.set_mouse_cursor(cursor),
            MaybeCefBrowser::Uninitialized { .. } => {}
        }
    }

    #[allow(dead_code)] // We never call this in win/linux currently.
    pub(crate) fn set_ime_position(&mut self, pos: Vec2) {
        match self {
            MaybeCefBrowser::Initialized(cef_browser) => cef_browser.set_ime_position(pos),
            MaybeCefBrowser::Uninitialized { .. } => {}
        }
    }

    #[allow(dead_code)] // We never call this in win/linux currently.
    pub(crate) fn return_to_js(&mut self, callback_id: u32, mut params: Vec<ZapParam>) {
        params.insert(0, format!("{}", callback_id).into_param());
        self.call_js("_zaplibReturnParams", params);
    }
}

#[derive(Debug)]
pub(crate) struct CallJsEvent {
    pub(crate) name: String,
    pub(crate) params: Vec<ZapParam>,
}

struct MyRenderProcessHandler {
    receive_channel: Arc<mpsc::Receiver<CallJsEvent>>,
    /// Synchronization mechanism to signal when JS init code was called
    ready_to_process: Arc<AtomicBool>,
    call_rust_sync_fn: Arc<RwLock<Option<CallRustSyncFn>>>,
}

fn make_mutable_buffer<T>(param_type: u32, mut buffer: Vec<T>) -> V8Value {
    let ptr = buffer.as_mut_ptr();
    let len = buffer.len();
    let callback = Arc::new(MyV8ArrayBufferReleaseCallback { buffer: Arc::new(buffer) });

    let buffer_data = V8Value::create_array(3);

    let v8_buffer = V8Value::create_array_buffer(ptr as *const u8, len * std::mem::size_of::<T>(), callback);
    buffer_data.set_value_byindex(0, &v8_buffer);

    // Purposefully leave the second array element blank to set arc_ptr as undefined

    buffer_data.set_value_byindex(2, &V8Value::create_uint(param_type));

    buffer_data
}

fn make_readonly_buffer<T>(param_type: u32, buffer: Arc<Vec<T>>) -> V8Value {
    let callback = Arc::new(MyV8ArrayBufferReleaseCallback { buffer: Arc::clone(&buffer) });

    let buffer_data = V8Value::create_array(3);

    let v8_buffer = V8Value::create_array_buffer(buffer.as_ptr() as *const u8, buffer.len() * std::mem::size_of::<T>(), callback);
    buffer_data.set_value_byindex(0, &v8_buffer);

    let arc_ptr = V8Value::create_uint(Arc::as_ptr(&buffer) as u32);
    buffer_data.set_value_byindex(1, &arc_ptr);

    buffer_data.set_value_byindex(2, &V8Value::create_uint(param_type));

    buffer_data
}

fn make_buffers_and_arc_ptrs(params: Vec<ZapParam>) -> V8Value {
    let values = V8Value::create_array(params.len());

    for (index, param) in params.into_iter().enumerate() {
        let param_type = match &param {
            ZapParam::String(_) => ZAP_PARAM_STRING,
            ZapParam::ReadOnlyU8Buffer(_) => ZAP_PARAM_READ_ONLY_UINT8_BUFFER,
            ZapParam::MutableU8Buffer(_) => ZAP_PARAM_UINT8_BUFFER,
            ZapParam::ReadOnlyF32Buffer(_) => ZAP_PARAM_READ_ONLY_FLOAT32_BUFFER,
            ZapParam::MutableF32Buffer(_) => ZAP_PARAM_FLOAT32_BUFFER,
        };
        let value = match param {
            ZapParam::String(str) => V8Value::create_string(&str),
            ZapParam::ReadOnlyU8Buffer(buffer) => make_readonly_buffer(param_type, buffer),
            ZapParam::MutableU8Buffer(buffer) => make_mutable_buffer(param_type, buffer),
            ZapParam::ReadOnlyF32Buffer(buffer) => make_readonly_buffer(param_type, buffer),
            ZapParam::MutableF32Buffer(buffer) => make_mutable_buffer(param_type, buffer),
        };

        values.set_value_byindex(index, &value);
    }
    values
}

fn get_zap_params(array: &V8Value) -> Vec<ZapParam> {
    assert!(array.is_array());
    let mut params = Vec::new();
    for index in 0..array.get_array_length() {
        let param = array.get_value_byindex(index).unwrap();

        let zap_param = if param.is_string() {
            param.get_string_value().into_param()
        } else {
            let array_buffer = param.get_value_byindex(0).unwrap();
            let param_type = param.get_value_byindex(1).unwrap().get_int_value() as u32;

            // TODO(Paras): Figure out a way to transfer ownership of mutable buffers without copying.
            // This copy is okay in the short term while we think of CEF as a dev-tool.
            // TODO(Paras): Figure out a way to extract this to a generic function to avoid code duplication :(
            match param_type {
                ZAP_PARAM_READ_ONLY_UINT8_BUFFER => {
                    let callback =
                        array_buffer.get_array_buffer_release_callback::<MyV8ArrayBufferReleaseCallback<u8>>().unwrap();
                    Arc::clone(&callback.buffer).into_param()
                }
                ZAP_PARAM_UINT8_BUFFER => {
                    let callback =
                        array_buffer.get_array_buffer_release_callback::<MyV8ArrayBufferReleaseCallback<u8>>().unwrap();
                    callback.buffer.to_vec().into_param()
                }
                ZAP_PARAM_FLOAT32_BUFFER => {
                    let callback =
                        array_buffer.get_array_buffer_release_callback::<MyV8ArrayBufferReleaseCallback<f32>>().unwrap();
                    callback.buffer.to_vec().into_param()
                }
                ZAP_PARAM_READ_ONLY_FLOAT32_BUFFER => {
                    let callback =
                        array_buffer.get_array_buffer_release_callback::<MyV8ArrayBufferReleaseCallback<f32>>().unwrap();
                    Arc::clone(&callback.buffer).into_param()
                }
                v => panic!("Invalid param type: {}", v),
            }
        };
        params.push(zap_param);
    }
    params
}

/// Process [`CallJsEvent`] messages by calling the appropriate `from_cef_js_functions` function in JS.
fn process_messages(receive_channel: &mpsc::Receiver<CallJsEvent>, frame: &Frame) {
    while let Ok(data) = receive_channel.try_recv() {
        let context = frame.get_v8context().unwrap();

        // Values could only be set within opened V8 context
        assert!(context.enter());
        let object = context.get_global().unwrap();
        let transformed_params = make_buffers_and_arc_ptrs(data.params);
        let obj_name = V8Value::create_string(&data.name);
        // TODO(Dmitry): switch to execute_function_with_context and pass all these variables inside context instead
        let id = universal_rand::random_128();
        let key_name = format!("name_{}", id);
        let key_params = format!("params_{}", id);
        assert!(object.set_value_bykey(&key_name, &obj_name, V8PropertyAttribute::V8_PROPERTY_ATTRIBUTE_NONE));
        assert!(object.set_value_bykey(&key_params, &transformed_params, V8PropertyAttribute::V8_PROPERTY_ATTRIBUTE_NONE));
        let code = format!(
            r#"
            window.fromCefCallJsFunction(window.{}, window.{});
            delete window.{}; delete window.{};
            "#,
            key_name, key_params, key_name, key_params,
        );
        assert!(context.exit());
        let script_url = "".to_string();
        let start_line = 0;
        frame.execute_javascript(&code, &script_url, start_line);
    }
}

struct MyV8ArrayBufferReleaseCallback<T> {
    #[allow(dead_code)]
    buffer: Arc<Vec<T>>,
}

// The memory is managed automatically when MyV8ArrayBufferReleaseCallback goes out of scope,
// this will deallocate self.buffer and decrement corresponding refcount
// This relies on assumption that inside CEF the callback is deallocated soon after release_buffer is called:
// see https://github.com/chromiumembedded/cef/blob/62a9f00bd3/libcef/renderer/v8_impl.cc#L1411-L1414
impl<T> V8ArrayBufferReleaseCallback for MyV8ArrayBufferReleaseCallback<T> {}

fn create_array_buffer<T: Default + Clone>(count: i32) -> V8Value {
    let callback = Arc::new(MyV8ArrayBufferReleaseCallback::<T> { buffer: Arc::new(vec![T::default(); count as usize]) });

    let value = V8Value::create_array(2);
    let arc_ptr = V8Value::create_uint(Arc::as_ptr(&callback.buffer) as u32);
    let v8_buffer = V8Value::create_array_buffer(
        callback.buffer.as_ptr() as *const u8,
        callback.buffer.len() * std::mem::size_of::<T>(),
        callback,
    );
    value.set_value_byindex(0, &v8_buffer);
    value.set_value_byindex(1, &arc_ptr);
    value
}

impl RenderProcessHandler for MyRenderProcessHandler {
    fn on_process_message_received(
        &self,
        _browser: &Browser,
        frame: &Frame,
        _source_process: ProcessId,
        _message: &ProcessMessage,
    ) -> bool {
        // Process messages only after browser was fully initialized.
        // Otherwise there will be errors when trying to access from_cef_js_functions property
        if self.ready_to_process.load(Ordering::Relaxed) {
            process_messages(&self.receive_channel, frame);
        }
        true
    }

    fn on_context_created(&self, _browser: &Browser, frame: &Frame, context: &V8Context) {
        let window = context.get_global().unwrap();

        // Connect `window.cefCallRustAsync` to `SystemEvent::WebRustCall` on the main thread.
        // Note: This is also used in runtime detection for `jsRuntime`. If this is renamed or
        // removed, that must be updated.
        assert!(window.set_fn_value("cefCallRustAsync", (), |_name, _obj, args, _other_data| {
            let name = args[0].get_string_value();
            let params = get_zap_params(&args[1]);
            let callback_id = args[2].get_uint_value();
            Cx::send_event_from_any_thread(Event::System(SystemEvent::WebRustCall(Some(WebRustCallEvent {
                name,
                params,
                callback_id,
            }))));
            None
        }));

        // Connect `window.cefCallRustSync` to `call_rust_sync_fn`
        // on whatever thread this is getting called from.
        assert!(window.set_fn_value(
            "cefCallRustSync",
            self.call_rust_sync_fn.clone(),
            |_name, _obj, args, call_rust_sync_fn| {
                let name = args[0].get_string_value();
                let params = get_zap_params(&args[1]);

                if let Some(func) = *call_rust_sync_fn.read().unwrap() {
                    let return_buffers = Cx::call_rust_sync_dispatch(func, name, params);
                    Some(Ok(make_buffers_and_arc_ptrs(return_buffers)))
                } else {
                    panic!("call_rust_sync was called without call_rust_sync_fn being set");
                }
            }
        ));

        // When `window.cefReadyForMessages` gets called in JS, call `process_messages` to flush any queued up messages.
        assert!(window.set_fn_value(
            "cefReadyForMessages",
            (Arc::clone(&self.receive_channel), frame.clone(), self.ready_to_process.clone()),
            |_, _, _, (receive_channel, frame, ready_to_process)| {
                ready_to_process.store(true, Ordering::Relaxed);
                process_messages(receive_channel, frame);
                None
            }
        ));

        assert!(window.set_fn_value("cefCreateArrayBuffer", (), |_, _, args, _other_data| {
            if args.is_empty() {
                return Some(Err("Undefined buffer size".to_string()));
            }

            let count = args[0].get_int_value();
            let param_type = args[1].get_int_value() as u32;
            let value = match param_type {
                ZAP_PARAM_READ_ONLY_UINT8_BUFFER | ZAP_PARAM_UINT8_BUFFER => create_array_buffer::<u8>(count),
                ZAP_PARAM_FLOAT32_BUFFER | ZAP_PARAM_READ_ONLY_FLOAT32_BUFFER => create_array_buffer::<f32>(count),
                v => panic!("Invalid param type: {}", v),
            };

            Some(Ok(value))
        }));

        assert!(window.set_fn_value("cefHandleKeyboardEvent", (), |_name, _obj, args, _other_data| {
            let callback = args[0].get_array_buffer_release_callback::<MyV8ArrayBufferReleaseCallback<u8>>();
            let buffer = &callback.unwrap().buffer;
            let mut zerde_parser = ZerdeParser::from(buffer.as_ptr() as u64);
            let msg_type = zerde_parser.parse_u32();
            let event = parse_keyboard_event_from_js(msg_type, &mut zerde_parser);
            Cx::send_event_from_any_thread(event);
            None
        }));

        assert!(window.set_fn_value("cefTriggerCut", frame.clone(), |_name, _obj, _args, frame| {
            frame.cut();
            None
        }));
        assert!(window.set_fn_value("cefTriggerCopy", frame.clone(), |_name, _obj, _args, frame| {
            frame.copy();
            None
        }));
        assert!(window.set_fn_value("cefTriggerPaste", frame.clone(), |_name, _obj, _args, frame| {
            frame.paste();
            None
        }));
        assert!(window.set_fn_value("cefTriggerSelectAll", frame.clone(), |_name, _obj, _args, frame| {
            frame.select_all();
            None
        }));
    }
}

struct MyBrowserProcessHandler {}
impl BrowserProcessHandler for MyBrowserProcessHandler {
    fn on_schedule_message_pump_work(&self, delay_ms: i64) {
        // See https://bitbucket.org/chromiumembedded/cef/issues/2968/documentation-of-external-message-pump
        Cx::cef_schedule_message_pump_work(delay_ms);
    }
}

struct MyApp {
    render_process_handler: Arc<MyRenderProcessHandler>,
    browser_process_handler: Arc<MyBrowserProcessHandler>,
}

impl App for MyApp {
    type OutBrowserProcessHandler = MyBrowserProcessHandler;
    type OutRenderProcessHandler = MyRenderProcessHandler;

    fn on_before_command_line_processing(&self, _process_type: &str, command_line: &CommandLine) {
        // Disable the macOS keychain prompt. Cookies will not be encrypted.
        #[cfg(target_os = "macos")]
        command_line.append_switch("use-mock-keychain");

        // We use the `--single-process` flag in Chromium, which the docs say is not very well supported,
        // but which seems to be (kind of) used in Android Webviews anyway so it might not be too bad.
        // If we don't do this we will get different processes for managing the window vs rendering, which
        // will require a major overhaul of zaplib.
        command_line.append_switch("single-process");
    }

    fn get_render_process_handler(&self) -> Option<Arc<Self::OutRenderProcessHandler>> {
        Some(self.render_process_handler.clone())
    }

    fn get_browser_process_handler(&self) -> Option<Arc<Self::OutBrowserProcessHandler>> {
        Some(self.browser_process_handler.clone())
    }
}

struct MyContextMenuHandler {}

impl ContextMenuHandler for MyContextMenuHandler {
    fn on_before_context_menu(&self, _browser: &Browser, _frame: &Frame, model: &MenuModel) {
        // Disabling right-click for now based on this approach https://magpcss.org/ceforum/viewtopic.php?f=6&t=15712
        model.clear();
    }
}

pub type GetResourceUrlCallback = fn(&str, &str) -> String;

struct MyResourceHandler {
    url: RwLock<String>,
    mime_type: RwLock<Option<String>>,
    contents: RwLock<Option<BufReader<File>>>,
    get_resource_url_callback: Option<GetResourceUrlCallback>,
}

impl ResourceHandler for MyResourceHandler {
    fn open(&self, url: &str) -> bool {
        if self.contents.read().unwrap().is_some() {
            panic!("Already loading this file!!!")
        }

        let url = {
            if let Some(callback) = self.get_resource_url_callback {
                #[cfg(not(all(target_os = "macos", feature = "cef-server")))]
                let current_directory = "".to_string();

                #[cfg(all(target_os = "macos", feature = "cef-server"))]
                let current_directory = get_bundle_directory();

                callback(url, &current_directory)
            } else {
                url.to_string()
            }
        };

        *self.mime_type.write().unwrap() = match Path::new(&url).extension().unwrap().to_str().unwrap() {
            "html" => Some("text/html".to_string()),
            "js" => Some("text/javascript".to_string()),
            "css" => Some("text/css".to_string()),
            "wasm" => Some("application/wasm".to_string()),
            "ico" => Some("image/vnd.microsoft.icon".to_string()),
            "bin" => Some("application/octet-stream".to_string()),
            _ => {
                println!("Unknown mime type for url {}", &url);
                None
            }
        };

        *self.contents.write().unwrap() = match File::open(&url) {
            Ok(f) => Some(BufReader::new(f)),
            Err(e) => {
                println!("Cannot read file: {}, {}", &url, &e);
                None
            }
        };

        *self.url.write().unwrap() = url;

        self.contents.read().unwrap().is_some()
    }

    fn get_mime_type(&self) -> Option<String> {
        self.mime_type.read().unwrap().clone()
    }

    fn get_status_code(&self) -> i32 {
        if self.contents.read().unwrap().is_some() {
            200
        } else {
            404
        }
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        if let Some(contents) = self.contents.write().unwrap().as_mut() {
            contents.read(buf)
        } else {
            Err(Error::new(ErrorKind::Unsupported, "Unsupported operation"))
        }
    }
}

struct MyResourceRequestHandler {
    resource_handler: Arc<MyResourceHandler>,
}

impl ResourceRequestHandler for MyResourceRequestHandler {
    type OutResourceHandler = MyResourceHandler;

    fn get_resource_handler(&self) -> Option<Arc<Self::OutResourceHandler>> {
        Some(self.resource_handler.clone())
    }
}

struct MyRequestHandler {
    handlers: RwLock<Vec<Arc<MyResourceRequestHandler>>>,
    get_resource_url_callback: Option<GetResourceUrlCallback>,
}

impl RequestHandler for MyRequestHandler {
    type OutResourceRequestHandler = MyResourceRequestHandler;

    fn get_resource_request_handler(&self, url: &str) -> Option<Arc<Self::OutResourceRequestHandler>> {
        let handler = Arc::new(MyResourceRequestHandler {
            resource_handler: Arc::new(MyResourceHandler {
                url: RwLock::new(url.to_string()),
                mime_type: RwLock::new(None),
                contents: RwLock::new(None),
                get_resource_url_callback: self.get_resource_url_callback,
            }),
        });

        self.handlers.write().unwrap().push(handler.clone());
        Some(handler)
    }
}

struct MyClient {
    context_menu_handler: Arc<MyContextMenuHandler>,
    #[cfg(feature = "cef-server")]
    request_handler: Arc<MyRequestHandler>,
}
impl Client for MyClient {
    type OutContextMenuHandler = MyContextMenuHandler;
    type OutRequestHandler = MyRequestHandler;

    fn get_context_menu_handler(&self) -> Option<Arc<Self::OutContextMenuHandler>> {
        Some(self.context_menu_handler.clone())
    }

    fn get_request_handler(&self) -> Option<Arc<Self::OutRequestHandler>> {
        #[cfg(not(feature = "cef-server"))]
        {
            None
        }

        #[cfg(feature = "cef-server")]
        Some(self.request_handler.clone())
    }
}

pub(crate) struct CefBrowser {
    pub(crate) browser: Arc<Browser>,
    call_rust_sync_fn: Arc<RwLock<Option<CallRustSyncFn>>>,
    send_channel: mpsc::Sender<CallJsEvent>,
}

impl CefBrowser {
    #[allow(dead_code)] // We never initialize in win/linux currently.
    fn new(
        size: Vec2,
        url: &str,
        parent_window: WindowHandle,
        (tx, rx): (mpsc::Sender<CallJsEvent>, mpsc::Receiver<CallJsEvent>),
        #[cfg(feature = "cef-server")] get_resource_url_callback: Option<GetResourceUrlCallback>,
    ) -> Self {
        let call_rust_sync_fn = Arc::new(RwLock::new(None));

        let app = Arc::new(MyApp {
            render_process_handler: Arc::new(MyRenderProcessHandler {
                receive_channel: Arc::new(rx),
                ready_to_process: Arc::new(AtomicBool::new(false)),
                call_rust_sync_fn: Arc::clone(&call_rust_sync_fn),
            }),
            browser_process_handler: Arc::new(MyBrowserProcessHandler {}),
        });
        let result_code = execute_process(&app);

        if result_code != -1 {
            process::exit(0);
        }

        let mut settings = Settings::default();
        let framework_dir_path = env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("Frameworks")
            .join("Chromium Embedded Framework.framework");
        settings.framework_dir_path = Some(framework_dir_path.to_str().unwrap());

        // TODO(JP): Having to set a dummy.app to not crash on assertions seems silly.. There must be a better way, right?
        #[allow(unused_variables)]
        let main_bundle_path = env::current_exe().unwrap().parent().unwrap().join("dummy.app");
        #[cfg(feature = "cef-debug")]
        {
            if !main_bundle_path.exists() {
                std::fs::create_dir(&main_bundle_path).unwrap();
                std::fs::create_dir(&main_bundle_path.join("Contents")).unwrap();
                std::fs::write(
                    &main_bundle_path.join("Contents").join("Info.plist"),
                    r#"
                <?xml version="1.0" encoding="UTF-8"?>
                <!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN""http://www.apple.com/DTDs/PropertyList-1.0.dtd">
                <plist version="1.0">
                <dict>
                <key>CFBundleExecutable</key>
                <string>dummy</string>
                <key>CFBundleIdentifier</key>
                <string>com.dummy</string>
                <key>CFBundleInfoDictionaryVersion</key>
                <string>6.0</string>
                <key>CFBundleName</key>
                <string>Dummy</string>
                <key>CFBundleShortVersionString</key>
                <string>0.0.1</string>
                <key>CFBundleVersion</key>
                <string>20210625.235709</string>
                </dict>
                </plist>
                "#,
                )
                .unwrap();
            }
            settings.main_bundle_path = Some(main_bundle_path.to_str().unwrap());
        }

        settings.external_message_pump = true;
        settings.log_file = Some("cef.log");
        #[cfg(feature = "cef-debug")]
        {
            settings.log_severity = zaplib_cef::LogSeverity::LOGSEVERITY_VERBOSE;
        }
        initialize(settings, &app);

        let window_info = WindowInfo { width: size.x as u32, height: size.y as u32, parent_window, ..Default::default() };

        // classic zaplib grey color
        let browser_settings = BrowserSettings { background_color: vec4_to_cef_color(&Vec4::color("3")), ..Default::default() };

        let client = Arc::new(MyClient {
            context_menu_handler: Arc::new(MyContextMenuHandler {}),
            #[cfg(feature = "cef-server")]
            request_handler: Arc::new(MyRequestHandler { handlers: RwLock::new(vec![]), get_resource_url_callback }),
        });
        let browser = Arc::new(create_browser_sync(window_info, &client, url, browser_settings));

        #[cfg(feature = "cef-dev-tools")]
        browser.get_host().unwrap().show_dev_tools();

        Self { browser, send_channel: tx, call_rust_sync_fn }
    }

    fn call_js(&mut self, call_js_event: CallJsEvent) {
        if self.send_channel.send(call_js_event).is_err() {
            log!("Error while sending CallJsEvent");
        }

        let frame = self.browser.get_main_frame().unwrap();
        let message = ProcessMessage::create("call_js");
        frame.send_process_message(ProcessId::PID_RENDERER, &message);
    }

    fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        let code = format!("if (window.fromCefSetMouseCursor) window.fromCefSetMouseCursor({});", cursor as u8);
        let script_url = "".to_string();
        let start_line = 0;
        let frame = self.browser.get_main_frame().unwrap();
        frame.execute_javascript(&code, &script_url, start_line);
    }

    fn set_ime_position(&mut self, pos: Vec2) {
        let code = format!("if (window.fromCefSetIMEPosition) window.fromCefSetIMEPosition({}, {});", pos.x, pos.y);
        let script_url = "".to_string();
        let start_line = 0;
        let frame = self.browser.get_main_frame().unwrap();
        frame.execute_javascript(&code, &script_url, start_line);
    }

    fn on_call_rust_sync(&mut self, func: CallRustSyncFn) {
        *self.call_rust_sync_fn.write().unwrap() = Some(func);
    }
}

impl Cx {
    /// See https://bitbucket.org/chromiumembedded/cef/issues/2968/documentation-of-external-message-pump
    #[allow(dead_code)] // We never initialize in win/linux currently.
    pub(crate) fn cef_do_message_loop_work(&mut self) {
        if let MaybeCefBrowser::Initialized(_) = self.cef_browser {
            zaplib_cef::do_message_loop_work();
        }
    }
}
