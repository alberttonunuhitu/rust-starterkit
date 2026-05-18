use actix_web::{HttpRequest, HttpResponse};
use serde_json::json;

use crate::common::response::ApiResponse;

pub async fn welcome(request: HttpRequest) -> HttpResponse {
    let data = json!({
        "message": "Rust starter kit",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running",
    });

    HttpResponse::Ok().json(ApiResponse::success(data, &request))
}
