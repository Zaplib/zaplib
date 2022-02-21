use actix_files::Files;
use actix_web::{middleware, rt, App as ActixApp, HttpServer};
use log::info;
use openssl::{
    pkey::PKey,
    ssl::{SslAcceptor, SslMethod},
    x509::X509,
};
use rcgen::generate_simple_self_signed;

pub(crate) fn serve(path: String, port: u16, ssl: bool) {
    let server_future = server_thread(path, port, ssl);
    rt::System::new().block_on(server_future)
}

async fn server_thread(path: String, port: u16, ssl: bool) {
    info!("Static server of '{path}' starting on port {port}");
    // srv is server controller type, `dev::Server`
    let mut http_server = HttpServer::new(move || {
        ActixApp::new()
            // enable logger
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
    });

    if ssl {
        info!("Generating self-signed certificates");
        let cert = generate_simple_self_signed(vec!["bs-local.com".to_string()]).unwrap();
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder.set_private_key(&PKey::private_key_from_pem(cert.serialize_private_key_pem().as_bytes()).unwrap()).unwrap();
        builder.set_certificate(&X509::from_pem(cert.serialize_pem().unwrap().as_bytes()).unwrap()).unwrap();
        http_server = http_server.bind_openssl(format!("0.0.0.0:{}", port), builder).unwrap();
    } else {
        http_server = http_server.bind(("0.0.0.0", port)).unwrap();
    }

    let server = http_server.workers(2).run();
    let protocol = if ssl { "https" } else { "http" };
    info!("Serving on {}://localhost:{}", protocol, port);
    server.await.unwrap();
}
