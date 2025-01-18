use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppData,
    err::{Error, ErrorType},
    err_from_type, err_with_context,
};

use super::{
    controller::{
        generate_access_token, get_private_key, get_public_key, get_refresh_token, validate_data,
        validate_login_request,
    },
    utils::double_decrypt,
};

pub struct NewDeviceQuery {
    device_name: String,
    device_public_key: Vec<u8>,
}

/// Get the public key of the server
#[axum::debug_handler]
pub async fn handshake(
    State(app_data): State<AppData>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let public_key = get_public_key(&app_data.data_dir).await?;
    let body = Body::from(public_key.to_vec());
    Ok(public_key)
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password_hash: String,
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

/// Register a new device
/// Read the public key of the device, and the login request data double encrypted
/// Recieve access and refresh token
#[axum::debug_handler]
pub async fn register(
    State(app_data): State<AppData>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let device_public_key = Vec::from(&body[0..512]);
    let server_public_key = get_private_key(&app_data.data_dir).await?;
    let login_request_encrypted = Vec::from(&body[512..]);
    let login_request = double_decrypt::<LoginRequest>(
        &server_public_key,
        &device_public_key,
        login_request_encrypted,
    )
    .await?;

    if validate_login_request(&app_data.data_dir, &login_request).await? {
        let access_token = generate_access_token(&device_public_key).await?;
        let refresh_token = get_refresh_token(&device_public_key).await?;
        let login_response = LoginResponse {
            access_token,
            refresh_token,
        };
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(
                serde_json::to_string(&login_response)
                    .map_err(|e| err_with_context!(e, "Failed to serialize response"))?,
            )
            .map_err(|e| err_with_context!(e, "Failed to build response"))?)
    } else {
        Err(err_from_type!(ErrorType::Unauthorized))
    }
}

/// Get the access token in case the refresh token is active
#[axum::debug_handler]
pub async fn get_access_token(
    State(data): State<AppData>,
    body: String,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let login_response = validate_data(&data.data_dir, &body).await?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(
            serde_json::to_string(&login_response)
                .map_err(|e| err_with_context!(e, "Failed to serialize response"))?,
        )
        .map_err(|e| err_with_context!(e, "Failed to build response"))?)
}
