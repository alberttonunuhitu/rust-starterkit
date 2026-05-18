use std::task::{Context, Poll};

use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{self, HeaderValue},
};

use futures_util::future::{LocalBoxFuture, Ready, ok};
use uuid::Uuid;

use crate::{common::constants::X_REQUEST_ID_HEADER, utils::uuid::generate_uuidv7};

#[derive(Debug, Clone, Copy)]
pub struct RequestId(pub Uuid);

pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestIdMiddlewareService { service })
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(context)
    }

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let id = generate_uuidv7();

        request.extensions_mut().insert(RequestId(id));

        let future = self.service.call(request);

        Box::pin(async move {
            let mut response = future.await?;
            let value = HeaderValue::from_str(&id.to_string())
                .expect("UUID is always a valid header value");

            response
                .headers_mut()
                .insert(header::HeaderName::from_static(X_REQUEST_ID_HEADER), value);

            Ok(response)
        })
    }
}
