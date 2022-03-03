//! To run locally on macOS:
//! * $ brew install --cask chromedriver
//! * $ chromedriver

use std::{env, error::Error, fs, path::Path, sync::mpsc, thread};

use actix_files::Files;
use actix_web::{dev::ServerHandle, middleware, rt, App as ActixApp, HttpServer};
use clap::{Arg, Command};
use futures::future::join_all;
use log::{error, info};
use openssl::{
    pkey::PKey,
    ssl::{SslAcceptor, SslMethod},
    x509::X509,
};
use rcgen::generate_simple_self_signed;
use serde_json::json;
use simple_error::SimpleError;
use thirtyfour::{Capabilities, DesiredCapabilities, WebDriver};

pub(crate) fn cmd() {
    // Use "info" logging level by default.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let matches = Command::new("Zaplib Continuous Integration (CI) Tool")
        .arg_required_else_help(true)
        .about(env!["CARGO_PKG_DESCRIPTION"])
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("webdriver-url")
                .long("webdriver-url")
                .takes_value(true)
                .help("HTTP(S) URL to connect to the Selenium Webdriver to"),
        )
        .arg(
            Arg::new("browserstack-local-identifier")
                .long("browserstack-local-identifier")
                .takes_value(true)
                .help("Local identifier for Browserstack"),
        )
        .get_matches();

    // Arbitrary port that we don't use elsewhere.
    // We start a server so the browser can access our files.
    let local_port = 1122;

    // Create a "screenshots" directory if it doesn't already exist.
    fs::create_dir_all("screenshots").unwrap();

    let (tx, rx) = mpsc::channel();
    let server_thread = thread::spawn(move || {
        let server_future = server_thread(tx, ".".to_string(), local_port);
        rt::System::new().block_on(server_future)
    });
    let server_handle = rx.recv().unwrap();

    rt::System::new().block_on(run_tests(
        matches.value_of("webdriver-url").unwrap().to_string(),
        local_port,
        matches.value_of("browserstack-local-identifier"),
    ));

    rt::System::new().block_on(server_handle.stop(true));
    server_thread.join().unwrap();
}

async fn run_tests(webdriver_url: String, local_port: u16, browserstack_local_identifier: Option<&str>) {
    if let Some(browserstack_local_identifier) = browserstack_local_identifier {
        // Uncomment Firefox and Safari once we get them working.
        // See https://github.com/Zaplib/zaplib/issues/67
        let mut capabilities_set = json!({
            "OS X Monterey, Chrome": {
                "bstack:options" : {
                    "os" : "OS X",
                    "osVersion" : "Monterey",
                    "consoleLogs": "verbose",
                },
                "browserName" : "Chrome",
                "browserVersion" : "98.0",
            },
            // "OS X Monterey, Firefox": {
            //     "bstack:options" : {
            //         "os" : "OS X",
            //         "osVersion" : "Monterey",
            //     },
            //     "browserName" : "Firefox",
            //     "browserVersion" : "latest",
            // },
            // "OS X Monterey, Safari": {
            //     "bstack:options" : {
            //         "os" : "OS X",
            //         "osVersion" : "Monterey",
            //     },
            //     "browserName" : "Safari",
            //     "browserVersion" : "latest",
            // },
            "OS X Monterey, Edge": {
                "bstack:options" : {
                    "os" : "OS X",
                    "osVersion" : "Monterey",
                },
                "browserName" : "Edge",
                "browserVersion" : "98.0",
            },
            "Windows 11, Chrome": {
                "bstack:options" : {
                    "os" : "Windows",
                    "osVersion" : "11",
                    "consoleLogs": "verbose",
                },
                "browserName" : "Chrome",
                "browserVersion" : "98.0",
            },
            // "Windows 11, Firefox": {
            //     "bstack:options" : {
            //         "os" : "Windows",
            //         "osVersion" : "11",
            //     },
            //     "browserName" : "Firefox",
            //     "browserVersion" : "latest",
            // },
            "Windows 11, Edge": {
                "bstack:options" : {
                    "os" : "Windows",
                    "osVersion" : "11",
                },
                "browserName" : "Edge",
                "browserVersion" : "98.0",
            },
            // "iPhone 13, iOS 15": {
            //     "device" : "iPhone 13",
            //     "osVersion" : "15",
            //     "browserName" : "iPhone",
            // },
            "Samsung Galaxy S21, Android 11.0": {
                "bstack:options" : {
                    "osVersion" : "11.0",
                    "deviceName" : "Samsung Galaxy S21",
                    "appiumVersion" : "1.22.0",
                    "consoleLogs": "verbose",
                },
                "browserName" : "Android",
            },
        });
        let futures: Vec<_> = capabilities_set
            .as_object_mut()
            .unwrap()
            .iter()
            .map(|(browser_name, capabilities_json)| {
                let mut capabilities = DesiredCapabilities::new(capabilities_json.clone());
                capabilities.add("acceptSslCerts", true).unwrap();
                capabilities.add_subkey("bstack:options", "projectName", "Zaplib").unwrap();
                capabilities
                    .add_subkey(
                        "bstack:options",
                        "buildName",
                        env::var("GITHUB_REF").unwrap_or_else(|_| "(no git branch)".to_string())
                            + " -- "
                            + &env::var("GITHUB_SHA").unwrap_or_else(|_| "(no git sha)".to_string()),
                    )
                    .unwrap();
                capabilities.add_subkey("bstack:options", "sessionName", &browser_name).unwrap();
                capabilities.add_subkey("bstack:options", "local", "true").unwrap();
                capabilities.add_subkey("bstack:options", "networkLogs", "true").unwrap();
                capabilities.add_subkey("bstack:options", "seleniumVersion", "3.5.2").unwrap();
                capabilities.add_subkey("bstack:options", "localIdentifier", browserstack_local_identifier).unwrap();
                let webdriver_url_str = webdriver_url.as_str();
                async move {
                    match WebDriver::new(webdriver_url_str, &capabilities).await {
                        Err(err) => {
                            error!("[{browser_name}] Connection error: {err}");
                            false
                        }
                        Ok(mut driver) => {
                            let result = match test_suite_all_tests_3x(browser_name, &mut driver, local_port).await {
                                Err(err) => {
                                    error!("[{browser_name}] Run error: {err}");
                                    false
                                }
                                Ok(()) => {
                                    // TODO(JP): Samsung Galaxy is a bit unstable and crashes throughout the session;
                                    // enable it later. See https://github.com/Zaplib/zaplib/issues/67
                                    if browser_name == "Samsung Galaxy S21, Android 11.0" {
                                        true
                                    } else {
                                        match examples_screenshots(browser_name, &mut driver, local_port).await {
                                            Err(err) => {
                                                error!("[{browser_name}] Run error: {err}");
                                                false
                                            }
                                            Ok(()) => true,
                                        }
                                    }
                                }
                            };
                            if result {
                                driver
                                    .execute_script(
                                        r#"browserstack_executor: {"action": "setSessionStatus", "arguments":
                                            {"status": "passed", "reason": ""}}"#,
                                    )
                                    .await
                                    .unwrap();
                            } else {
                                driver
                                    .execute_script(
                                        r#"browserstack_executor: {"action": "setSessionStatus", "arguments":
                                            {"status": "failed", "reason": ""}}"#,
                                    )
                                    .await
                                    .unwrap();
                            }
                            driver.quit().await.unwrap();
                            result
                        }
                    }
                }
            })
            .collect();
        for result in join_all(futures).await {
            if !result {
                panic!("At least one test failed");
            }
        }
    } else {
        let mut capabilities = DesiredCapabilities::new(json!({}));
        capabilities.add("acceptSslCerts", true).unwrap();
        let mut driver = WebDriver::new(&webdriver_url, &capabilities).await.unwrap();
        test_suite_all_tests_3x("local browser", &mut driver, local_port).await.unwrap();
        driver.quit().await.unwrap();
    }
}

async fn test_suite_all_tests_3x(browser_name: &str, driver: &mut WebDriver, local_port: u16) -> Result<(), Box<dyn Error>> {
    info!("[{browser_name}] Connected to WebDriver...");
    // bs-local.com redirects to localhost; necessary for using HTTPS with Browserstack.
    driver.get(format!("https://bs-local.com:{}/zaplib/web/test_suite", local_port)).await?;
    info!("[{browser_name}] Running tests...");
    info!("[{browser_name}] For console output see the browser/Browserstack directly. \
        See https://github.com/stevepryde/thirtyfour/issues/87");
    let script = r#"
        const done = arguments[0];
        const interval = setInterval(() => {
            if (window.runAllTests3x) {
                clearInterval(interval);
                window.runAllTests3x().then(() => done('SUCCESS'), (err) => done(err.stack));
            }
        }, 10);
    "#;
    let result = driver.execute_async_script(script).await?;
    driver.screenshot(Path::new(&("screenshots/test_suite_all_tests_3x --".to_string() + browser_name + ".png"))).await?;
    match result.value().as_str().unwrap_or("--zaplib_ci: no string was returned--") {
        "SUCCESS" => {
            info!("[{browser_name}] Tests passed!");
            Ok(())
        }
        str => Err(Box::new(SimpleError::new(format!("Tests failed: {str}")))),
    }
}

async fn examples_screenshots(browser_name: &str, driver: &mut WebDriver, local_port: u16) -> Result<(), Box<dyn Error>> {
    let examples = [
        // Tracking these TODOs in https://github.com/Zaplib/zaplib/issues/29
        // "example_bigedit/?release", // TODO(JP): Pause animation.
        // ("example_charts", "example_charts/?release"), // TODO(JP): Randomness.
        // "example_lightning", // TODO(JP): Pause animation.
        ("example_lots_of_buttons", "example_lots_of_buttons/?release"),
        ("example_shader", "example_shader/?release"),
        ("example_single_button", "example_single_button/?release"),
        ("example_text", "example_text/?release"),
        ("test_bottom_bar", "test_bottom_bar/?release"),
        // ("test_geometry", "test_geometry/?release"), // TODO(JP): Pause animation.
        ("test_layout", "test_layout/?release"),
        // "test_many_quads/?release", // TODO(JP): Pause animation.
        // "test_multithread/?release", // TODO(JP): Pause animation.
        ("test_padding", "test_padding/?release"),
        ("test_popover", "test_popover/?release"),
        // "test_shader_2d_primitives/", // TODO(JP): Make work in Wasm context (not just CEF).
        ("tutorial_2d_rendering_step1", "tutorial_2d_rendering/step1"),
        ("tutorial_2d_rendering_step2", "tutorial_2d_rendering/step2"),
        ("tutorial_2d_rendering_step3", "tutorial_2d_rendering/step3"),
        ("tutorial_3d_rendering_step2", "tutorial_3d_rendering/step2"),
        ("tutorial_3d_rendering_step3", "tutorial_3d_rendering/step3"),
        ("tutorial_hello_thread", "tutorial_hello_thread"),
        ("tutorial_hello_world_canvas", "tutorial_hello_world_canvas"),
        ("tutorial_hello_world_console", "tutorial_hello_world_console"),
        ("tutorial_js_rust_bridge", "tutorial_js_rust_bridge"),
        ("tutorial_ui_components", "tutorial_ui_components"),
        ("tutorial_ui_layout", "tutorial_ui_layout"),
    ];

    for (example_name, example_path) in examples {
        let url = format!("https://bs-local.com:{}/zaplib/examples/{}", local_port, example_path);
        info!("[{browser_name}] Navigating to {url}...");
        driver.get(url).await?;
        let script = r#"
            const done = arguments[0];
            const interval = setInterval(() => {
                if (zaplib.isInitialized()) {
                    clearInterval(interval);
                    setTimeout(() => {
                        done("SUCCESS");
                    }, 3000); // TODO(JP): Shorten this time. See https://github.com/Zaplib/zaplib/issues/29
                }
            }, 10);
        "#;
        let result = driver.execute_async_script(script).await?;
        driver.screenshot(Path::new(&("screenshots/".to_string() + example_name + " --" + browser_name + ".png"))).await?;
        match result.value().as_str().unwrap_or("--zaplib_ci: no string was returned--") {
            "SUCCESS" => {
                info!("[{browser_name}] Successfully taken screenshot of {example_name}");
            }
            str => return Err(Box::new(SimpleError::new(format!("Screenshot {example_name} failed: {str}")))),
        }
    }
    Ok(())
}

/// NOTE(JP): There is some overlap with the code for `cargo zaplib serve`, but they might diverge. If these
/// evolve in a way where it makes sense to share code, then we should look into refactoring this.
async fn server_thread(tx: mpsc::Sender<ServerHandle>, path: String, port: u16) {
    info!("Generating self-signed certificates");
    let cert = generate_simple_self_signed(vec!["localhost".to_string(), "bs-local.com".to_string()]).unwrap();
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key(&PKey::private_key_from_pem(cert.serialize_private_key_pem().as_bytes()).unwrap()).unwrap();
    builder.set_certificate(&X509::from_pem(cert.serialize_pem().unwrap().as_bytes()).unwrap()).unwrap();

    info!("Static HTTPS server of '{path}' starting on port {port}");
    let server = HttpServer::new(move || {
        ActixApp::new()
            .wrap(middleware::Logger::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Cross-Origin-Opener-Policy", "same-origin"))
                    .add(("Cross-Origin-Embedder-Policy", "require-corp"))
                    .add(("Access-Control-Allow-Origin", "*")),
            )
            .service(
                Files::new("/", &path)
                    .show_files_listing()
                    .index_file("index.html")
                    .use_etag(true)
                    .use_last_modified(true)
                    .redirect_to_slash_directory()
                    .use_hidden_files(),
            )
    })
    .bind_openssl(format!("0.0.0.0:{}", port), builder)
    .unwrap()
    .workers(2)
    .run();

    tx.send(server.handle()).unwrap();

    info!("Serving on https://localhost:{}", port);
    server.await.unwrap();
}
