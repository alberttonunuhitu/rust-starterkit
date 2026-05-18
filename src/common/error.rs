use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

use super::response::{Meta, extract_request_id};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

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

    #[error("Bad request")]
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
