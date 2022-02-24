//! Common code between all native desktop platforms. The counterpart is `cx_wasm32`.

use crate::*;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;

#[derive(Clone, Default)]
pub(crate) struct CxDesktop {
    pub(crate) repaint_via_scroll_event: bool,
}

impl CxDesktopVsWasmCommon for Cx {
    /// See [`CxDesktopVsWasmCommon::get_default_window_size`] for documentation.
    fn get_default_window_size(&self) -> Vec2 {
        Vec2 { x: 800., y: 600. }
    }

    /// See [`CxDesktopVsWasmCommon::file_write`] for documentation.
    fn file_write(&mut self, path: &str, data: &[u8]) {
        // just write it right now
        if let Ok(mut file) = File::create(path) {
            if file.write_all(data).is_ok() {
            } else {
                println!("ERROR WRITING FILE {}", path);
            }
        } else {
            println!("ERROR WRITING FILE {}", path);
        }
    }

    /// See [`CxDesktopVsWasmCommon::websocket_send`] for documentation.
    fn websocket_send(&mut self, _url: &str, _data: &[u8]) {}

    /// See [`CxDesktopVsWasmCommon::http_send`] for documentation.
    fn http_send(
        &mut self,
        verb: &str,
        path: &str,
        _proto: &str,
        domain: &str,
        port: u16,
        content_type: &str,
        body: &[u8],
        signal: Signal,
    ) {
        fn write_bytes_to_tcp_stream(tcp_stream: &mut TcpStream, bytes: &[u8]) -> bool {
            let bytes_total = bytes.len();
            let mut bytes_left = bytes_total;
            while bytes_left > 0 {
                let buf = &bytes[(bytes_total - bytes_left)..bytes_total];
                if let Ok(bytes_written) = tcp_stream.write(buf) {
                    if bytes_written == 0 {
                        return false;
                    }
                    bytes_left -= bytes_written;
                } else {
                    return true;
                }
            }
            false
        }

        // start a thread, connect, and report back.
        let data = body.to_vec();
        let byte_len = data.len();
        let header = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nConnect: close\r\nContent-Type:{}\r\nContent-Length:{}\r\n\r\n",
            verb, path, domain, content_type, byte_len
        );
        let host = format!("{}:{}", domain, port);
        let _connect_thread = {
            std::thread::spawn(move || {
                let stream = TcpStream::connect(&host);
                if let Ok(mut stream) = stream {
                    if !write_bytes_to_tcp_stream(&mut stream, header.as_bytes())
                        && !write_bytes_to_tcp_stream(&mut stream, &data)
                    {
                        Cx::post_signal(signal, Cx::STATUS_HTTP_SEND_OK);
                        return;
                    }
                }
                Cx::post_signal(signal, Cx::STATUS_HTTP_SEND_FAIL);
            })
        };
    }

    /// See [`CxDesktopVsWasmCommon::call_js`] for documentation.
    #[allow(unused_variables)]
    #[cfg(feature = "cef")]
    fn call_js(&mut self, name: &str, params: Vec<ZapParam>) {
        self.cef_browser.call_js(name, params);
    }

    /// See [`CxDesktopVsWasmCommon::return_to_js`] for documentation.
    #[cfg(feature = "cef")]
    fn return_to_js(&mut self, callback_id: u32, params: Vec<ZapParam>) {
        self.cef_browser.return_to_js(callback_id, params);
    }
    /// This never gets called if cef is not enabled, but we need it to pass compilation.
    #[cfg(not(feature = "cef"))]
    fn return_to_js(&mut self, _callback_id: u32, _params: Vec<ZapParam>) {}
}

impl Cx {
    pub(crate) fn process_desktop_paint_callbacks(&mut self) -> bool {
        let mut vsync = false; //self.platform.desktop.repaint_via_scroll_event;
        self.platform.desktop.repaint_via_scroll_event = false;
        if self.requested_next_frame {
            self.call_next_frame_event();
            if self.requested_next_frame {
                vsync = true;
            }
        }

        self.call_signals();

        // call redraw event
        if self.requested_draw {
            self.call_draw_event();
        }
        if self.requested_draw {
            vsync = true;
        }

        self.call_signals();

        vsync
    }

    /// See [`Cx::on_call_rust_sync`] for documentation.
    #[cfg(feature = "cef")]
    pub(crate) fn on_call_rust_sync_internal(&mut self, func: CallRustSyncFn) {
        self.cef_browser.on_call_rust_sync(func);
    }
}
