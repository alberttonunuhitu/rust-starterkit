use super::handler::welcome;
use actix_web::web;

pub fn health_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(welcome));
}
