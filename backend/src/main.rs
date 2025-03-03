#![feature(macro_metavar_expr)]

use actix_cors::Cors;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{
    App, HttpServer,
    cookie::{Key, time::Duration},
    web,
};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;

mod db;
mod handlers;
mod macros;
mod models;
mod routes;
mod utils;

use db::{Cache, Db};

struct State {
    db: Db,
    cache: Cache,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addrs: &str;
    if cfg!(debug_assertions) {
        addrs = "localhost";
        println!("Development Server running at http://{}:1234", addrs);
    } else {
        addrs = "0.0.0.0";
        println!("Production Server running at https://{}:1234", addrs);
    }

    let key = Key::generate();

    let (db, cache) = match Db::new().await {
        Ok((db, cache)) => (db, cache),
        Err(e) => {
            eprintln!("Failed to connect to the database: {:?}", e);
            return Ok(());
        }
    };

    let state = web::Data::new(State { db, cache });

    #[cfg(feature = "log")]
    const LOG_LEVEL: &str = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };

    #[cfg(feature = "log")]
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(LOG_LEVEL));

    if cfg!(feature = "log") {
        HttpServer::new(move || {
            let session_middleware = if cfg!(debug_assertions) {
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(false)
                    .cookie_http_only(false)
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
                    )
                    .build()
            } else {
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::None)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
                    )
                    .build()
            };

            App::new()
                .configure(routes::init)
                .wrap(Cors::permissive())
                .wrap(session_middleware)
                .wrap(actix_web::middleware::Compress::default())
                .wrap(actix_web::middleware::Logger::default())
                .app_data(state.clone())
        })
        .bind((addrs, 1234))?
        .run()
        .await
    } else {
        HttpServer::new(move || {
            let session_middleware = if cfg!(debug_assertions) {
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(false)
                    .cookie_http_only(false)
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
                    )
                    .build()
            } else {
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::None)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
                    )
                    .build()
            };

            App::new()
                .configure(routes::init)
                .wrap(Cors::permissive())
                .wrap(session_middleware)
                .wrap(actix_web::middleware::Compress::default())
                .app_data(state.clone())
        })
        .bind((addrs, 1234))?
        .run()
        .await
    }
}
