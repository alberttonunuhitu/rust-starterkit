use actix_web::{App, HttpServer, web};
use anyhow::Result;
use rust_starterkit::config::AppConfig;
use rust_starterkit::state::AppState;
use rust_starterkit::{
    common::middleware::{authenticate::AuthenticateMiddleware, request_id::RequestIdMiddleware},
    infrastructure::database::connection::establish_connection,
};
use tracing_actix_web::TracingLogger;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env().map_err(std::io::Error::other)?;

    let _log_guard = init_tracing(&config.log.dir).map_err(std::io::Error::other)?;

    tracing::info!(
        host = %config.server.host,
        port = config.server.port,
        "Starting server"
    );

    rust_starterkit::utils::paseto::init(&config).map_err(std::io::Error::other)?;

    let db = establish_connection(&config)
        .await
        .map_err(std::io::Error::other)?;

    let bind = (config.server.host.clone(), config.server.port);
    let state = web::Data::new(AppState::new(db, config));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(RequestIdMiddleware)
            .wrap(TracingLogger::default())
            .configure(public_routes)
            .configure(protected_routes)
    })
    .bind(bind)?
    .run()
    .await
}

fn public_routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(rust_starterkit::features::health::routes::health_routes)
        .configure(rust_starterkit::features::auth::routes::auth_routes);
}

fn protected_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(AuthenticateMiddleware)
            .configure(rust_starterkit::features::user::routes::user_routes)
            .configure(rust_starterkit::features::auth::routes::protected_auth_routes),
    );
}

fn init_tracing(log_dir: &str) -> Result<WorkerGuard> {
    std::fs::create_dir_all(log_dir)?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "rust-starterkit.log");
    let (writer, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .with_span_list(true)
        .flatten_event(true)
        .with_writer(writer);

    let console_layer = fmt::layer().pretty().with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    Ok(guard)
}
