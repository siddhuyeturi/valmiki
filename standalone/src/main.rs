use actix_web::{
    App, HttpServer,
    cookie::Key,
    middleware::{Compress, Logger},
    web,
};
use base64::Engine;
use bb8::Pool;
use log::info;
use redis::Client;
use valmiki_visitors::VisitorMiddleware;

#[tokio::main]
async fn main() {
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
    // redis url
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL env variable not set");
    let client = Client::open(redis_url.as_str()).expect("Failed to create a redis client");
    let redis_pool = Pool::builder()
        .build(client)
        .await
        .expect("Failed to create a redis bb8 connection pool");
    info!("http://localhost:8080/");
    HttpServer::new(move || {
        App::new()
            .app_data(web::ThinData(redis_pool.clone()))
            .wrap(VisitorMiddleware::new(key.clone()))
            .wrap(Logger::default())
            .wrap(Compress::default())
            .service(index)
    })
    .bind_auto_h2c(("127.0.0.1", 8080))
    .expect("Failed to bind to 127.0.0.1:8080")
    .run()
    .await
    .expect("Failed to run server")
}

#[actix_web::get("/")]
pub async fn index() -> &'static str {
    "Hello, world!"
}
