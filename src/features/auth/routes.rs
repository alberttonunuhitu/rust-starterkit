use super::handler::{login, logout, refresh_token};
use crate::common::middleware::authenticate::AuthenticateMiddleware;
use actix_web::web;

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh_token))
            .service(
                web::resource("/logout")
                    .wrap(AuthenticateMiddleware)
                    .route(web::post().to(logout)),
            ),
    );
}
