use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
};

use crate::{app::AppData, err::Error, err_with_context};

use super::implementation::{get_public_key, get_token};

pub struct NewDeviceQuery {
    device_name: String,
    device_public_key: Vec<u8>,
}

pub async fn handshake(State(data): State<AppData>) -> Result<impl IntoResponse, Error> {
    println!("Getting public key");
    let public_key = get_public_key(&data.data_dir).await?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(public_key))
        .map_err(|e| err_with_context!(e, "Building response for public key"))
}

pub async fn new_device(
    State(data): State<AppData>,
    data_stream: Bytes,
) -> Result<impl IntoResponse, Error> {
    let data_stream = data_stream.to_vec();
    let jwt_token = get_token(&data.data_dir, data_stream).await?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(jwt_token))
        .map_err(|e| err_with_context!(e, "Building response for new device"))
}

pub async fn device_login(
    State(data): State<AppData>,
    data_stream: Bytes,
) -> Result<impl IntoResponse, Error> {
    let data_stream = data_stream.to_vec();
    let jwt_token = get_token(&data.data_dir, data_stream).await?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(jwt_token))
        .map_err(|e| err_with_context!(e, "Building response for device login"))
}
