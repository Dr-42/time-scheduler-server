use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::{app::AppState, err::Error, err_with_context};

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    req: axum::http::Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let headers = req.headers();
    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(bearer_token) = auth_str.strip_prefix("Bearer ") {
                if bearer_token == app_state.password_hash.as_str() {
                    return Ok(next.run(req).await);
                }
            }
        }
    }
    println!("Unauthorized request");
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from("Unauthorized"))
        .map_err(|e| err_with_context!(e, "Building for unauthorized request"))
}
