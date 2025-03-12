use std::{fs::File, io::BufReader};

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
use rustls::{ServerConfig, pki_types::PrivateKeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys};

struct State {
    db: Db,
    cache: Cache,
}

#[derive(argh::FromArgs)]
/// Start the server
struct CliArgs {
    /// the port to run the server on
    #[argh(option, short = 'p', default = "1234")]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: CliArgs = argh::from_env();
    let port = args.port;
    let addrs: &str;
    if cfg!(debug_assertions) {
        addrs = "localhost";
        println!("Development Server running at http://{}:{}", addrs, port);
    } else {
        addrs = "0.0.0.0";
        println!("Production Server running at https://{}:{}", addrs, port);
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

    const LOG_LEVEL: &str = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(LOG_LEVEL));

    // TODO: See the cookie warning
    if cfg!(debug_assertions) {
        HttpServer::new(move || {
            App::new()
                .configure(routes::init)
                .wrap(Cors::permissive())
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                        .cookie_secure(false)
                        .cookie_http_only(false)
                        .cookie_same_site(actix_web::cookie::SameSite::Strict)
                        .session_lifecycle(
                            PersistentSession::default()
                                .session_ttl(Duration::seconds(SECS_IN_WEEK)),
                        )
                        .build(),
                )
                .wrap(actix_web::middleware::Compress::default())
                .wrap(actix_web::middleware::Logger::default())
                .app_data(state.clone())
        })
        .bind((addrs, port))?
        .run()
        .await
    } else {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .unwrap();

        let config = ServerConfig::builder().with_no_client_auth();

        let cert_file = &mut BufReader::new(File::open("certs/cert.pem").unwrap());
        let key_file = &mut BufReader::new(File::open("certs/key.pem").unwrap());

        let cert_chain = certs(cert_file).collect::<Result<Vec<_>, _>>().unwrap();
        let mut keys = pkcs8_private_keys(key_file)
            .map(|key| key.map(PrivateKeyDer::Pkcs8))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        if keys.is_empty() {
            eprintln!("Could not locate PKCS 8 private keys.");
            std::process::exit(1);
        }

        let config = config.with_single_cert(cert_chain, keys.remove(0)).unwrap();

        HttpServer::new(move || {
            App::new()
                .configure(routes::init)
                .wrap(Cors::permissive())
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                        .cookie_secure(true)
                        .cookie_http_only(true)
                        .cookie_same_site(actix_web::cookie::SameSite::None)
                        .session_lifecycle(
                            PersistentSession::default()
                                .session_ttl(Duration::seconds(SECS_IN_WEEK)),
                        )
                        .build(),
                )
                .wrap(actix_web::middleware::Compress::default())
                .wrap(actix_web::middleware::Logger::default())
                .app_data(state.clone())
        })
        .bind_rustls_0_23(&format!("0.0.0.0:{}", port), config)?
        .run()
        .await
    }
}
