use std::task::{Context, Poll};

use actix_web::{
    Error, HttpMessage, ResponseError,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use uuid::Uuid;

use crate::common::error::ApiError;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: String,
}

pub struct AuthenticateMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthenticateMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticateMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticateMiddlewareService { service })
    }
}

pub struct AuthenticateMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticateMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(context)
    }

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let token = extract_bearer_token(&request);

        let Some(token) = token else {
            let (req, _) = request.into_parts();
            let response = ApiError::Unauthorized
                .error_response()
                .map_into_right_body();
            return Box::pin(async { Ok(ServiceResponse::new(req, response)) });
        };

        match crate::utils::paseto::verify_access_token(&token) {
            Err(e) => {
                tracing::warn!(event = "auth.middleware.token_invalid", error = %e);
                let (req, _) = request.into_parts();
                let response = ApiError::Unauthorized
                    .error_response()
                    .map_into_right_body();
                Box::pin(async { Ok(ServiceResponse::new(req, response)) })
            }
            Ok(claims) => match extract_user_from_claims(&claims) {
                Err(e) => {
                    tracing::warn!(event = "auth.middleware.claims_invalid", error = %e);
                    let (req, _) = request.into_parts();
                    let response = ApiError::Unauthorized
                        .error_response()
                        .map_into_right_body();
                    Box::pin(async { Ok(ServiceResponse::new(req, response)) })
                }
                Ok(user) => {
                    request.extensions_mut().insert(user);
                    let future = self.service.call(request);
                    Box::pin(async move { future.await.map(|res| res.map_into_left_body()) })
                }
            },
        }
    }
}

fn extract_bearer_token(request: &ServiceRequest) -> Option<String> {
    let value = request.headers().get(header::AUTHORIZATION)?;
    value
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::to_owned)
}

fn extract_user_from_claims(
    claims: &pasetors::claims::Claims,
) -> anyhow::Result<AuthenticatedUser> {
    let id_str = claims
        .get_claim("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing sub claim"))?;

    let id = Uuid::parse_str(id_str).map_err(|_| anyhow::anyhow!("invalid UUID in sub claim"))?;

    let email = claims
        .get_claim("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing email claim"))?
        .to_owned();

    Ok(AuthenticatedUser { id, email })
}
