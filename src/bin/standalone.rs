use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer,
    cookie::Key,
    middleware::{Compress, Logger},
    web::{self, Bytes},
};
use base64::Engine;
use log::info;
use valmiki::visitors::VisitorMiddleware;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // init logger
    pretty_env_logger::init();
    // load secret key from env to sign/encrypt cookies
    let key = std::env::var("SECRET_KEY")
        .map(|v| {
            let decoded_key = base64::engine::general_purpose::STANDARD
                .decode(v.as_bytes())
                .expect("failed to decode secret key from env var");
            Key::derive_from(&decoded_key)
        })
        .expect("failed to read secret key from env var");
    info!("http://127.0.0.1:8080/");
    HttpServer::new(move || {
        App::new()
            .wrap(VisitorMiddleware::new(key.clone()))
            .wrap(Logger::default())
            .wrap(Compress::default())
            .default_service(web::to(default_handler))
    })
    .bind_auto_h2c(("127.0.0.1", 8080))
    .map_err(|e| anyhow::anyhow!("Failed to bind h2c on 127.0.0.1:8080 because {e}"))?
    .run()
    .await?;

    Ok(())
}

pub async fn default_handler(_req: HttpRequest, _payload: Bytes) -> HttpResponse {
    HttpResponse::NotFound().body("Hello, World!")
}
