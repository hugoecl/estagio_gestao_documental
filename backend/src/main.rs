// TODO: See about running the schema on build.rs
// TODO: Benchmark web::Bytes and sonic vs web::Json and serde_json

#[cfg(feature = "https")]
use std::{fs::File, io::BufReader};

use actix_cors::Cors;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{
    App, HttpServer,
    cookie::{Key, time::Duration},
    web,
};
use cache::Cache;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;

mod auth;
mod cache;
mod db;
mod handlers;
mod macros;
mod models;
mod routes;
mod utils;

use db::Db;

#[cfg(feature = "https")]
use rustls::{ServerConfig, pki_types::PrivateKeyDer};
#[cfg(feature = "https")]
use rustls_pemfile::{certs, pkcs8_private_keys};

struct State {
    db: Db,
    cache: Cache,
}

#[derive(argh::FromArgs)]
/// Start the server
struct CliArgs {
    /// the address to run the server on (default: localhost on debug, 0.0.0.0 on release)
    #[argh(
        option,
        short = 'a',
        default = "if cfg!(debug_assertions) { String::from(\"localhost\") } else { String::from(\"0.0.0.0\") }"
    )]
    address: String,

    /// the port to run the server on (default: 1234)
    #[argh(option, short = 'p', default = "1234")]
    port: u16,

    #[cfg(feature = "https")]
    /// whether to use https
    #[argh(switch)]
    https: bool,

    #[cfg(feature = "https")]
    /// the path to the key file (default: certs/key.pem)
    #[argh(option, short = 'k', default = "String::from(\"certs/key.pem\")")]
    key_path: String,

    #[cfg(feature = "https")]
    /// the path to the cert file (default: certs/cert.pem)
    #[argh(option, short = 'c', default = "String::from(\"certs/cert.pem\")")]
    cert_path: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: CliArgs = argh::from_env();

    let (db, cache) = match Db::new().await {
        Ok(db) => match Cache::new(&db.pool).await {
            Ok(cache) => (db, cache),
            Err(e) => {
                eprintln!("Failed to create cache: {e:?}");
                return Ok(());
            }
        },
        Err(e) => {
            eprintln!("Failed to connect to the database: {e:?}");
            return Ok(());
        }
    };

    let state = web::Data::new(State { db, cache });

    #[cfg(feature = "https")]
    let protocol = if args.https { "https" } else { "http" };
    #[cfg(not(feature = "https"))]
    let protocol = "http";

    let log_level: &str;
    if cfg!(debug_assertions) {
        log_level = "debug";
        println!(
            "Development Server running at {}://{}:{}",
            protocol, args.address, args.port
        );
    } else {
        log_level = "warn";
        println!(
            "Production Server running at {}://{}:{}",
            protocol, args.address, args.port
        );
    }

    env_logger::init_from_env(env_logger::Env::new().default_filter_or(log_level));

    // TODO: See the cookie warning

    let key = Key::generate();
    let server = HttpServer::new(move || {
        let session_middleware: SessionMiddleware<CookieSessionStore> = if cfg!(debug_assertions) {
            SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                .cookie_secure(false)
                .cookie_http_only(false)
                .cookie_same_site(actix_web::cookie::SameSite::Strict)
                .session_lifecycle(
                    PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
                )
                .build()
        } else {
            let session_builder =
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone());

            #[cfg(feature = "https")]
            let session_builder = session_builder.cookie_secure(args.https);

            #[cfg(not(feature = "https"))]
            let session_builder = session_builder.cookie_secure(false);

            session_builder
                .cookie_http_only(true)
                .cookie_same_site(actix_web::cookie::SameSite::Lax)
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
    });
    #[cfg(feature = "https")]
    let result = if args.https {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .unwrap();

        let config = ServerConfig::builder().with_no_client_auth();

        let cert_file = &mut BufReader::new(File::open(args.cert_path).unwrap());
        let key_file = &mut BufReader::new(File::open(args.key_path).unwrap());

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
        server
            .bind_rustls_0_23(&format!("0.0.0.0:{}", args.port), config)?
            .run()
            .await
    } else {
        server.bind((args.address, args.port))?.run().await
    };

    #[cfg(not(feature = "https"))]
    let result = server.bind((args.address, args.port))?.run().await;

    result
}
