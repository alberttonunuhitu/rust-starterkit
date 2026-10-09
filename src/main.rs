use actix_web::{App, HttpServer, web};
use api::config::AppConfig;
use api::state::AppState;
use api::{
    common::{
        error::{json_error_handler, not_found, path_error_handler, query_error_handler},
        middleware::request_id::RequestIdMiddleware,
    },
    infrastructure::database::connection::establish_connection,
};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env().map_err(std::io::Error::other)?;

    init_tracing();

    tracing::info!(
        host = %config.server.host,
        port = config.server.port,
        "Starting server"
    );

    api::utils::paseto::init(&config).map_err(std::io::Error::other)?;

    let db = establish_connection(&config)
        .await
        .map_err(std::io::Error::other)?;

    let bind = (config.server.host.clone(), config.server.port);
    let state = web::Data::new(AppState::new(db, config));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .app_data(web::QueryConfig::default().error_handler(query_error_handler))
            .app_data(web::PathConfig::default().error_handler(path_error_handler))
            .wrap(RequestIdMiddleware)
            .wrap(TracingLogger::default())
            .configure(routes)
            .default_service(web::to(not_found))
    })
    .bind(bind)?
    .run()
    .await
}

fn routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(api::features::health::routes::health_routes)
        .configure(api::features::auth::routes::auth_routes)
        .configure(api::features::user::routes::user_routes);
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let console_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .with_span_list(true)
        .flatten_event(true)
        .with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .init();
}
