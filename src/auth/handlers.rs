use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use chrono::Duration;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppState,
    err::{Error, ErrorType},
    err_from_type, err_with_context,
};

use super::{
    controller::{verify_token, verify_user},
    middleware::TokenState,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub key: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub exp: usize,
}

#[axum_macros::debug_handler]
pub async fn login(
    Extension(state): Extension<AppState>,
    Json(login_info): Json<LoginRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if verify_user(&login_info, &state.password_hash) {
        let access_claims = Claims {
            exp: (chrono::Utc::now() + Duration::seconds(30)).timestamp() as usize,
        };

        let access_token = match encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(login_info.key.as_bytes()),
        ) {
            Ok(token) => token,
            Err(e) => {
                println!("Erron creating token: {}", e);
                return Err(err_with_context!(e, "Error creating access token"));
            }
        };

        let refresh_claims = Claims {
            exp: (chrono::Utc::now() + Duration::days(7)).timestamp() as usize,
        };

        let refresh_token = match encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(login_info.key.as_bytes()),
        ) {
            Ok(token) => token,
            Err(e) => {
                println!("Erron creating token: {}", e);
                return Err(err_with_context!(e, "Error creating refresh token"));
            }
        };

        Ok(Json(LoginResponse {
            access_token,
            refresh_token,
        }))
    } else {
        Err(err_from_type!(ErrorType::Unauthorized))
    }
}

#[axum_macros::debug_handler]
pub async fn refresh_token(
    Extension(state): Extension<AppState>,
    token: String,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.password_hash.as_bytes()),
        &Default::default(),
    ) {
        Ok(_) => {
            let access_claims = Claims {
                exp: (chrono::Utc::now() + Duration::seconds(30)).timestamp() as usize,
            };

            let access_token = encode(
                &Header::default(),
                &access_claims,
                &EncodingKey::from_secret(state.password_hash.as_bytes()),
            )
            .map_err(|e| err_with_context!(e, "Error creating access token"))?;

            let refresh_claims = Claims {
                exp: (chrono::Utc::now() + Duration::days(7)).timestamp() as usize,
            };

            let refresh_token = encode(
                &Header::default(),
                &refresh_claims,
                &EncodingKey::from_secret(state.password_hash.as_bytes()),
            )
            .map_err(|e| err_with_context!(e, "Error creating refresh token"))?;

            Ok(Json(LoginResponse {
                access_token,
                refresh_token,
            }))
        }
        Err(e) => {
            println!("Erron decoding token: {}", e);
            Err(err_from_type!(
                ErrorType::Unauthorized,
                "Unauthorized Access on token refresh"
            ))
        }
    }
}

#[axum_macros::debug_handler]
pub async fn check_token(
    Extension(state): Extension<AppState>,
    token: String,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match verify_token(&token, &state.password_hash) {
        Ok(token_state) => match token_state {
            TokenState::Valid => Ok(StatusCode::OK),
            TokenState::Expired => Err(Response::builder()
                .status(StatusCode::NETWORK_AUTHENTICATION_REQUIRED)
                .body(Body::from(
                    err_from_type!(ErrorType::TokenExpired, "Access token timed out").to_string(),
                ))
                .map_err(|e| err_with_context!(e, "Building for unauthorized request"))),
            TokenState::Unauthorized => Err(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from(
                    err_from_type!(ErrorType::Unauthorized, "Unauthorized request").to_string(),
                ))
                .map_err(|e| err_with_context!(e, "Building for unauthorized request"))),
        },
        Err(e) => Err(Err(e)),
    }
}
