#![deny(clippy::unwrap_used, clippy::expect_used)]

use app::{AppData, AppState};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Extension, Router,
};
use std::path::PathBuf;
use tokio::net::TcpListener;

mod analysis;
mod app;
mod auth;
mod blocktype;
mod currentblock;
mod err;
mod handlers;
mod timeblock;

pub async fn run(
    port: u16,
    password_hash: String,
    data_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let ip = format!("0.0.0.0:{}", port);
    let state = AppState::init(password_hash).await;
    let data = AppData::init(data_dir).await;

    let routes = Router::new()
        // Auth
        .route("/auth/login", post(auth::handlers::login))
        .route("/auth/refresh", post(auth::handlers::refresh_token))
        .layer(Extension(state.clone()))
        // Main home state for today
        .route("/state", get(handlers::get_entire_state))
        // Block types
        .route("/blocktype/get", get(handlers::get_blocktypes))
        .route("/blocktype/new", post(handlers::new_blocktype))
        // Time blocks
        .route("/timeblock/get", get(handlers::get_daydata))
        .route("/timeblock/next", post(handlers::next_timeblock))
        .route("/timeblock/split", post(handlers::split_timeblock))
        .route("/timeblock/adjust", post(handlers::adjust_timeblock))
        // Current block
        .route("/currentblock/get", get(handlers::get_current_block))
        .route("/currentblock/change", post(handlers::change_current_block))
        // Analysis
        .route("/analysis", get(handlers::get_analysis))
        .layer(from_fn_with_state(
            state.clone(),
            auth::middleware::auth_middleware,
        ))
        .with_state(data);

    let listener = TcpListener::bind(&ip).await?;
    axum::serve(listener, routes).await?;
    Ok(())
}
