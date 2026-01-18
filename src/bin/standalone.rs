use std::path::Path;

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
    let working_dir = std::env::var("WORKING_DIR")?;
    std::env::set_current_dir(&working_dir)?;
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|e| anyhow::anyhow!("DATABASE_URL env var is not set. source error: {e}"))?;
    let sqlite_db_file_path = Path::new(database_url.strip_prefix("sqlite://").ok_or(
        anyhow::anyhow!("DATABASE_URL is not in the format 'sqlite://main.db'"),
    )?);
    let db_exists = sqlite_db_file_path.exists();

    let sqlite_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(3)
        .min_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .foreign_keys(true)
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .filename(sqlite_db_file_path),
        )
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to create a sqlite pool at database_url {database_url} because {e}"
            )
        })?;

    if !db_exists {
        sqlx::migrate!("migrations/v1/sqlite")
            .run(&sqlite_pool)
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to migrate run migrations at source 'migrations/v1/sqlite' because {e}"
                )
            })?;
    }

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
