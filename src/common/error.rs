use std::collections::HashMap;

use actix_web::{
    HttpRequest, HttpResponse, ResponseError,
    error::{InternalError, JsonPayloadError, PathError, QueryPayloadError},
    http::StatusCode,
};
use serde::Serialize;
use thiserror::Error;

use super::response::{Meta, extract_request_id};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Resource not found")]
    NotFound,

    #[error("User not found")]
    UserNotFound,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Validation failed")]
    ValidationError {
        fields: HashMap<String, Vec<String>>,
    },

    #[error("Internal server error")]
    InternalServerError,

    #[error("{message}")]
    BadRequest { message: String },
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    success: bool,
    error: ErrorBody,
    meta: Meta,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<HashMap<String, Vec<String>>>,
}

impl ApiError {
    fn details(&self) -> (&'static str, StatusCode) {
        match self {
            Self::Unauthorized => ("UNAUTHORIZED", StatusCode::UNAUTHORIZED),

            Self::Forbidden => ("FORBIDDEN", StatusCode::FORBIDDEN),

            Self::NotFound => ("NOT_FOUND", StatusCode::NOT_FOUND),

            Self::UserNotFound => ("USER_NOT_FOUND", StatusCode::NOT_FOUND),

            Self::EmailAlreadyExists => ("EMAIL_ALREADY_EXISTS", StatusCode::CONFLICT),

            Self::ValidationError { .. } => ("VALIDATION_ERROR", StatusCode::UNPROCESSABLE_ENTITY),

            Self::InternalServerError => {
                ("INTERNAL_SERVER_ERROR", StatusCode::INTERNAL_SERVER_ERROR)
            }

            Self::BadRequest { .. } => ("BAD_REQUEST", StatusCode::BAD_REQUEST),
        }
    }

    fn fields(&self) -> Option<HashMap<String, Vec<String>>> {
        match self {
            Self::ValidationError { fields } => Some(fields.clone()),

            _ => None,
        }
    }

    fn build_response(&self, request_id: String) -> HttpResponse {
        let (code, status_code) = self.details();

        let body = ErrorResponse {
            success: false,
            error: ErrorBody {
                code,
                message: self.to_string(),
                fields: self.fields(),
            },
            meta: Meta::new(request_id),
        };

        HttpResponse::build(status_code).json(body)
    }

    pub fn to_response(&self, request: &HttpRequest) -> HttpResponse {
        self.build_response(extract_request_id(request))
    }

    /// Convert into an `actix_web::Error` whose response carries the request id.
    pub fn into_actix_error(self, request: &HttpRequest) -> actix_web::Error {
        let response = self.to_response(request);
        InternalError::from_response(self, response).into()
    }

    pub fn validation(field: &str, message: &str) -> Self {
        let mut fields = HashMap::new();

        fields.insert(field.to_string(), vec![message.to_string()]);

        Self::ValidationError { fields }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.details().1
    }

    fn error_response(&self) -> HttpResponse {
        self.build_response("n/a".to_string())
    }
}

pub type AppResult<T> = Result<T, ApiError>;

fn bad_request(error: impl std::fmt::Display, request: &HttpRequest) -> actix_web::Error {
    ApiError::BadRequest {
        message: error.to_string(),
    }
    .into_actix_error(request)
}

/// Render JSON body errors with the standard error envelope.
pub fn json_error_handler(error: JsonPayloadError, request: &HttpRequest) -> actix_web::Error {
    bad_request(error, request)
}

/// Render query string errors with the standard error envelope.
pub fn query_error_handler(error: QueryPayloadError, request: &HttpRequest) -> actix_web::Error {
    bad_request(error, request)
}

/// Render path parameter errors with the standard error envelope.
pub fn path_error_handler(error: PathError, request: &HttpRequest) -> actix_web::Error {
    bad_request(error, request)
}

/// Fallback for unmatched routes.
pub async fn not_found(request: HttpRequest) -> HttpResponse {
    ApiError::NotFound.to_response(&request)
}
