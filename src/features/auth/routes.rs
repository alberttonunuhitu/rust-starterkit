use super::handler::{login, logout, refresh_token};
use actix_web::web;

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh_token)),
    );
}

pub fn protected_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").route("/logout", web::post().to(logout)));
}
