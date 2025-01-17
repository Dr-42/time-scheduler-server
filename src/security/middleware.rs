use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::{
    app::AppData,
    err::{Error, ErrorType},
    err_from_type, err_with_context,
    security::implementation::verify_token,
};

pub async fn auth_middleware(
    State(app_state): State<AppData>,
    req: axum::http::Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let headers = req.headers();
    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(bearer_token) = auth_str.strip_prefix("Bearer ") {
                if verify_token(&app_state.data_dir, bearer_token).await? {
                    return Ok(next.run(req).await);
                }
            }
        }
    }
    println!("Unauthorized request");
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from(
            err_from_type!(ErrorType::Unauthorized, "Unauthorized request").to_string(),
        ))
        .map_err(|e| err_with_context!(e, "Building for unauthorized request"))
}
