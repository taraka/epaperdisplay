use std::sync::{Arc, Mutex};
use std::thread;

use crate::epd::paint::Image;

const DEFAULT_PORT: u16 = 8080;

const PAGE: &str = r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<title>Calendar</title>
<meta http-equiv="refresh" content="60">
<style>
  body { margin: 0; background: #222; display: flex; justify-content: center; align-items: center; min-height: 100vh; }
  img { max-width: 100%; height: auto; }
</style>
</head>
<body>
<img src="/image.bmp" alt="Calendar">
</body>
</html>
"#;

// Holds the most recently rendered frame as BMP bytes, shared between the
// main render loop and the HTTP server thread.
#[derive(Clone)]
pub struct Snapshot(Arc<Mutex<Vec<u8>>>);

impl Snapshot {
    pub fn new() -> Snapshot {
        Snapshot(Arc::new(Mutex::new(Vec::new())))
    }

    pub fn update(&self, image: &Image) {
        *self.0.lock().unwrap() = image.to_bmp();
    }

    fn get(&self) -> Vec<u8> {
        self.0.lock().unwrap().clone()
    }
}

pub fn start(snapshot: Snapshot) {
    let port = std::env::var("WEB_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    thread::spawn(move || {
        let server = match tiny_http::Server::http(("0.0.0.0", port)) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Web server failed to bind to port {}: {}", port, e);
                return;
            }
        };
        log::info!("Web server listening on http://0.0.0.0:{}", port);

        for request in server.incoming_requests() {
            let (body, content_type): (Vec<u8>, &str) = match request.url() {
                "/image.bmp" => (snapshot.get(), "image/bmp"),
                _ => (PAGE.as_bytes().to_vec(), "text/html; charset=utf-8"),
            };

            let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap();
            let response = tiny_http::Response::from_data(body).with_header(header);
            if let Err(e) = request.respond(response) {
                log::debug!("Web server failed to respond to request: {}", e);
            }
        }
    });
}
