#[cfg(feature = "https")]
use std::{fs::File, io::BufReader};

use actix_cors::Cors;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{
    App,
    HttpServer,
    cookie::{Key, time::Duration as ActixDuration},
    rt::spawn,
    web,
};
use actix_files::Files;
use mimalloc::MiMalloc;
use tokio::time::{Duration as TokioDuration, interval};

use crate::services::notification_service::check_expiring_date_ranges;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;

mod auth;
mod db;
mod handlers;
mod macros;
mod models;
mod routes;
mod services;
mod utils;

use db::Db;

#[cfg(feature = "https")]
use rustls::{ServerConfig, pki_types::PrivateKeyDer};
#[cfg(feature = "https")]
use rustls_pemfile::{certs, pkcs8_private_keys};

struct State {
    db: Db,
}

#[derive(argh::FromArgs)]
/// Start the server
struct CliArgs {
    /// address to run server
    #[argh(option, short = 'a', default = "String::from(\"localhost\")")]
    address: String,

    /// port to run server
    #[argh(option, short = 'p', default = "1234")]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: CliArgs = argh::from_env();

    let db = match Db::new().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to connect to the database: {e:?}");
            return Ok(());
        }
    };

    let state = web::Data::new(State { db });

    let state_clone = state.clone();
    spawn(async move {
        let mut timer = interval(TokioDuration::from_secs(3600));
        loop {
            timer.tick().await;
            check_expiring_date_ranges(&state_clone.db.pool).await;
        }
    });

    let key = Key::generate();

    HttpServer::new(move || {
        let session_middleware: SessionMiddleware<CookieSessionStore> =
            SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                .cookie_secure(false)
                .session_lifecycle(
                    PersistentSession::default()
                        .session_ttl(ActixDuration::seconds(SECS_IN_WEEK)),
                )
                .build();

        App::new()
            .configure(routes::init)
            .wrap(Cors::permissive())
            .wrap(session_middleware)
            .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::Logger::default())
            .app_data(state.clone())
            .service(Files::new("/media", "media").show_files_listing())
    })
    .bind((args.address, args.port))?
    .run()
    .await
}
