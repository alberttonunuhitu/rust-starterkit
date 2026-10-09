use actix_web::{App, HttpResponse, http::StatusCode, test, web};
use api::common::{
    error::{json_error_handler, not_found},
    middleware::{authenticate::AuthenticateMiddleware, request_id::RequestIdMiddleware},
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Payload {
    #[allow(dead_code)]
    email: String,
}

async fn echo(_: web::Json<Payload>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn ok() -> HttpResponse {
    HttpResponse::Ok().finish()
}

macro_rules! app {
    () => {
        test::init_service(
            App::new()
                .app_data(web::JsonConfig::default().error_handler(json_error_handler))
                .wrap(RequestIdMiddleware)
                .route("/echo", web::post().to(echo))
                .service(
                    web::scope("/protected")
                        .wrap(AuthenticateMiddleware)
                        .route("", web::get().to(ok)),
                )
                .default_service(web::to(not_found)),
        )
        .await
    };
}

fn assert_envelope(body: &Value, code: &str, request_id: &str) {
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], code);
    assert_eq!(body["meta"]["request_id"], request_id);
}

#[actix_web::test]
async fn invalid_json_uses_error_envelope() {
    let app = app!();
    let request = test::TestRequest::post()
        .uri("/echo")
        .insert_header(("content-type", "application/json"))
        .set_payload("{}")
        .to_request();

    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let request_id = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let body: Value = test::read_body_json(response).await;
    assert_envelope(&body, "BAD_REQUEST", &request_id);
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("missing field `email`")
    );
}

#[actix_web::test]
async fn missing_token_uses_error_envelope() {
    let app = app!();
    let request = test::TestRequest::get().uri("/protected").to_request();

    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let request_id = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let body: Value = test::read_body_json(response).await;
    assert_envelope(&body, "UNAUTHORIZED", &request_id);
}

#[actix_web::test]
async fn unknown_route_uses_error_envelope() {
    let app = app!();
    let request = test::TestRequest::get().uri("/does-not-exist").to_request();

    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let request_id = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let body: Value = test::read_body_json(response).await;
    assert_envelope(&body, "NOT_FOUND", &request_id);
}
