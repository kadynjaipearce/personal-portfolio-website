mod config;
mod db;
mod error;
mod middleware;
mod models;
mod routes;
mod services;

use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, web, App, HttpServer};
use config::CONFIG;
use tera::Tera;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    db::init_db().await.expect("Failed to initialize database");

    // Initialize templates
    let tera = Tera::new("templates/**/*.html").expect("Failed to initialize Tera templates");

    // Session key
    let session_key = Key::from(CONFIG.session_secret.as_bytes());

    info!("Starting server at http://{}", CONFIG.server_addr());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .wrap(Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), session_key.clone())
                    .cookie_secure(false) // Set to true in production with HTTPS
                    .cookie_http_only(true)
                    .build(),
            )
            // API routes
            .configure(routes::api_routes)
            // Auth routes
            .configure(routes::auth_routes)
            // Admin routes
            .configure(routes::admin_routes)
            // Static files
            .service(Files::new("/static", "static").show_files_listing())
            // Public routes
            .configure(routes::public_routes)
            // 404 for any unmatched path
            .default_service(web::to(routes::not_found))
    })
    .bind(CONFIG.server_addr())?
    .run()
    .await
}
