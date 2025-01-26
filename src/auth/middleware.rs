use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::{
    app::AppState,
    auth::controller::verify_token,
    err_from_type, err_with_context,
    error::{Error, ErrorType},
};

pub enum TokenState {
    Valid,
    Expired,
    Unauthorized,
}

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    req: axum::http::Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let headers = req.headers();
    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(bearer_token) = auth_str.strip_prefix("Bearer ") {
                match verify_token(bearer_token, &app_state.password_hash)? {
                    TokenState::Valid => return Ok(next.run(req).await),
                    TokenState::Expired => {
                        return Response::builder()
                            .status(StatusCode::NETWORK_AUTHENTICATION_REQUIRED)
                            .body(Body::from(
                                err_from_type!(ErrorType::TokenExpired, "Access token timed out")
                                    .to_string(),
                            ))
                            .map_err(|e| err_with_context!(e, "Building for unauthorized request"))
                    }
                    TokenState::Unauthorized => {
                        return Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body(Body::from(
                                err_from_type!(ErrorType::Unauthorized, "Unauthorized request")
                                    .to_string(),
                            ))
                            .map_err(|e| err_with_context!(e, "Building for unauthorized request"))
                    }
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
