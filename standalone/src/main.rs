use actix_web::{App, HttpServer, cookie::Key, web::ReqData};
use base64::Engine;
use valmiki_visitors::{VisitorId, VisitorMiddleware};

#[tokio::main]
async fn main() {
    let key = std::env::var("SECRET_KEY")
        .map(|v| {
            let decoded_key = base64::engine::general_purpose::STANDARD
                .decode(v.as_bytes())
                .expect("failed to decode secret key from env var");
            Key::derive_from(&decoded_key)
        })
        .expect("failed to read secret key from env var");
    HttpServer::new(move || {
        App::new()
            .wrap(VisitorMiddleware::new(key.clone()))
            .service(index)
    })
    .bind_auto_h2c(("127.0.0.1", 8080))
    .expect("Failed to bind to 127.0.0.1:8080")
    .run()
    .await
    .expect("Failed to run server")
}

#[actix_web::get("/")]
pub async fn index(vid: ReqData<VisitorId>) -> &'static str {
    println!("vid: {}", vid.as_ref());
    "Hello, world!"
}
