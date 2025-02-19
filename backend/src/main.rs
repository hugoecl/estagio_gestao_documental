use actix_cors::Cors;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},
    App, HttpServer,
};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;

mod handlers;
mod routes;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if cfg!(debug_assertions) {
        println!("Development Server running at http://127.0.0.1:1234");
    } else {
        println!("Production Server running at https://0.0.0.0:1234");
    }

    let key = Key::generate();

    if cfg!(debug_assertions) {
        #[cfg(feature = "log")]
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    }

    HttpServer::new(move || {
        let session_middleware: SessionMiddleware<CookieSessionStore>;

        if cfg!(debug_assertions) {
            session_middleware =
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(false)
                    .cookie_http_only(false)
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
                    )
                    .build();
        } else {
            session_middleware =
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
                    )
                    .build();
        }
        App::new()
            .configure(routes::init)
            .wrap(Cors::permissive()) // TODO: Change this to a more secure configuration
            .wrap(actix_web::middleware::Logger::default())
            .wrap(session_middleware)
        // .app_data(state.clone())
        // .app_data(web::PayloadConfig::default().limit(1024 * 1024 * 5)) // 5 MB
    })
    .bind(("0.0.0.0", 1234))?
    .run()
    .await
}
