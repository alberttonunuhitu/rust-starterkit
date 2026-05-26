use crate::{
    common::{extractor::PathUuid, pagination::PaginatedResponse, response::ApiResponse},
    features::user::dto::{CreateUserRequest, UpdateUserRequest, UserPaginatedQuery, UserResponse},
    state::AppState,
};
use actix_web::{HttpRequest, HttpResponse, web};

pub async fn get_all_paginated_users(
    request: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<UserPaginatedQuery>,
) -> HttpResponse {
    match state
        .services
        .user
        .find_all_paginated(query.into_inner())
        .await
    {
        Ok((data, page, per_page, total)) => HttpResponse::Ok().json(PaginatedResponse::new(
            data, page, per_page, total, &request,
        )),
        Err(e) => e.to_response(&request),
    }
}

pub async fn create_user(
    request: HttpRequest,
    state: web::Data<AppState>,
    data: web::Json<CreateUserRequest>,
) -> HttpResponse {
    match state.services.user.create(&data).await {
        Ok(user) => {
            HttpResponse::Created().json(ApiResponse::success(UserResponse::from(user), &request))
        }
        Err(e) => e.to_response(&request),
    }
}

pub async fn get_user_by_id(
    request: HttpRequest,
    path: PathUuid,
    state: web::Data<AppState>,
) -> HttpResponse {
    let PathUuid(user_id) = path;

    match state.services.user.find_by_id(user_id).await {
        Ok(user) => {
            HttpResponse::Ok().json(ApiResponse::success(UserResponse::from(user), &request))
        }
        Err(e) => e.to_response(&request),
    }
}

pub async fn update_user(
    request: HttpRequest,
    path: PathUuid,
    state: web::Data<AppState>,
    data: web::Json<UpdateUserRequest>,
) -> HttpResponse {
    let PathUuid(user_id) = path;

    match state.services.user.update(user_id, &data).await {
        Ok(user) => {
            HttpResponse::Ok().json(ApiResponse::success(UserResponse::from(user), &request))
        }
        Err(e) => e.to_response(&request),
    }
}

pub async fn delete_user(
    request: HttpRequest,
    path: PathUuid,
    state: web::Data<AppState>,
) -> HttpResponse {
    let PathUuid(user_id) = path;

    match state.services.user.delete(user_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => e.to_response(&request),
    }
}
