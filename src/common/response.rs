use actix_web::{HttpMessage, HttpRequest};
use chrono::Utc;
use serde::Serialize;

use crate::common::middleware::request_id::RequestId;

#[derive(Debug, Serialize)]
pub struct Meta {
    pub request_id: String,
    pub timestamp: String,
}

impl Meta {
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: T,
    pub meta: Meta,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T, request: &HttpRequest) -> Self {
        Self {
            success: true,
            data,
            meta: Meta::new(extract_request_id(request)),
        }
    }
}

pub fn extract_request_id(request: &HttpRequest) -> String {
    request
        .extensions()
        .get::<RequestId>()
        .map(|r| r.0.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
