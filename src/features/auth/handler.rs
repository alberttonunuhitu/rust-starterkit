use super::dto::{LoginRequest, RefreshTokenRequest};
use crate::{
    common::{extractor::AuthUser, response::ApiResponse},
    state::AppState,
};
use actix_web::{HttpRequest, HttpResponse, web};

pub async fn login(
    request: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<LoginRequest>,
) -> HttpResponse {
    match state.services.auth.login(&payload).await {
        Ok(data) => HttpResponse::Ok().json(ApiResponse::success(data, &request)),
        Err(e) => e.to_response(&request),
    }
}

pub async fn refresh_token(
    request: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<RefreshTokenRequest>,
) -> HttpResponse {
    match state.services.auth.refresh_token(&payload).await {
        Ok(data) => HttpResponse::Ok().json(ApiResponse::success(data, &request)),
        Err(e) => e.to_response(&request),
    }
}

pub async fn logout(
    request: HttpRequest,
    state: web::Data<AppState>,
    auth_user: AuthUser,
) -> HttpResponse {
    match state.services.auth.logout(auth_user.0.id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => e.to_response(&request),
    }
}
