use actix_web::web;

use crate::common::middleware::authenticate::AuthenticateMiddleware;
use crate::features::user::handler::{
    create_user, delete_user, get_all_paginated_users, get_user_by_id, update_user,
};

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .wrap(AuthenticateMiddleware)
            .route("", web::get().to(get_all_paginated_users))
            .route("", web::post().to(create_user))
            .route("/{id}", web::get().to(get_user_by_id))
            .route("/{id}", web::put().to(update_user))
            .route("/{id}", web::delete().to(delete_user)),
    );
}
