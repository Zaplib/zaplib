//! To run locally on macOS:
//! * $ brew install --cask chromedriver
//! * $ chromedriver

use std::{sync::mpsc, thread};

use actix_files::Files;
use actix_web::{dev::ServerHandle, middleware, rt, App as ActixApp, HttpServer};
use clap::{Arg, Command};
use log::{error, info};
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
                .default_value("")
                .help("Local identifier for Browserstack"),
        )
        .get_matches();

    // Arbitrary port that we don't use elsewhere.
    // We start a server so the browser can access our files.
    let local_port = 1122;

    let (tx, rx) = mpsc::channel();
    let server_thread = thread::spawn(move || {
        let server_future = server_thread(tx, ".".to_string(), local_port);
        rt::System::new().block_on(server_future)
    });
    let server_handle = rx.recv().unwrap();

    rt::System::new().block_on(test_suite_all_tests_3x(
        matches.value_of("webdriver-url").unwrap().to_string(),
        local_port,
        matches.value_of("browserstack-local-identifier").unwrap().to_string(),
    ));

    rt::System::new().block_on(server_handle.stop(true));
    server_thread.join().unwrap();
}

async fn test_suite_all_tests_3x(webdriver_url: String, local_port: u16, browserstack_local_identifier: String) {
    let is_browserstack = webdriver_url.contains("browserstack.com");

    let mut caps = DesiredCapabilities::chrome();
    if is_browserstack {
        caps.add("browserstack.local", "true").unwrap();
        caps.add("browserstack.localIdentifier", &browserstack_local_identifier).unwrap();
    }

    let driver = WebDriver::new(&webdriver_url, &caps).await.unwrap();
    info!("Connected to WebDriver...");
    driver.get(format!("http://localhost:{}/zaplib/web/test_suite", local_port)).await.unwrap();
    info!("Running tests...");
    info!("For console output see the browser/Browserstack directly. See https://github.com/stevepryde/thirtyfour/issues/87");
    let script = r#"
        const done = arguments[0];
        const interval = setInterval(() => {
            if (window.runAllTests3x) {
                clearInterval(interval);
                window.runAllTests3x().then(() => done('SUCCESS'), (err) => done(err.stack));
            }
        }, 10);
    "#;
    match driver.execute_async_script(script).await.unwrap().value().as_str().unwrap() {
        "SUCCESS" => {
            info!("Tests passed!");
            if is_browserstack {
                driver
                    .execute_script(
                        r#"browserstack_executor: {"action": "setSessionStatus", "arguments":
                          {"status":"passed","reason": ""}}"#,
                    )
                    .await
                    .unwrap();
            }
        }
        str => {
            if is_browserstack {
                // Print test failure before we update Browserstack, in case that call fails.
                error!("Tests failed: {str}");
                driver
                    .execute_script(
                        r#"browserstack_executor: {"action": "setSessionStatus", "arguments":
                          {"status":"failed","reason": ""}}"#,
                    )
                    .await
                    .unwrap();
                panic!("Tests failed (see above)");
            } else {
                panic!("Tests failed: {str}");
            }
        }
    }
    driver.quit().await.unwrap();
}

/// NOTE(JP): There is some overlap with the code for `cargo zaplib serve`, but they might diverge. If these
/// evolve in a way where it makes sense to share code, then we should look into refactoring this.
async fn server_thread(tx: mpsc::Sender<ServerHandle>, path: String, port: u16) {
    info!("Static server of '{path}' starting on port {port}");
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
    .bind(("0.0.0.0", port))
    .unwrap()
    .workers(2)
    .run();

    tx.send(server.handle()).unwrap();

    info!("Serving on http://localhost:{}", port);
    server.await.unwrap();
}
