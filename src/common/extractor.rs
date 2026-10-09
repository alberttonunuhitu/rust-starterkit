use crate::{
    common::{error::ApiError, middleware::authenticate::AuthenticatedUser},
    utils::uuid::parse_uuid,
};
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest, dev::Payload};
use std::future::{Ready, ready};
use uuid::Uuid;

pub struct PathUuid(pub Uuid);

impl FromRequest for PathUuid {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, _: &mut Payload) -> Self::Future {
        let Some(id) = request.match_info().get("id") else {
            return ready(Err(ApiError::BadRequest {
                message: "Path parameter is required".to_string(),
            }
            .into_actix_error(request)));
        };

        ready(parse_uuid(id).map(PathUuid).map_err(|e| {
            tracing::warn!(event = "common.extractor.path_uuid.failed", error = %e);
            ApiError::BadRequest {
                message: "Invalid UUID format".to_string(),
            }
            .into_actix_error(request)
        }))
    }
}

pub struct AuthUser(pub AuthenticatedUser);

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, _: &mut Payload) -> Self::Future {
        let user = request.extensions().get::<AuthenticatedUser>().cloned();
        ready(
            user.map(AuthUser)
                .ok_or_else(|| ApiError::Unauthorized.into_actix_error(request)),
        )
    }
}
