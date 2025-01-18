#![deny(clippy::unwrap_used, clippy::expect_used)]

use app::{AppData, AppState};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    serve, Router,
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

    // NOTE: Main blocks. The basic api

    // Blocktype-related routes
    let blocktype_routes = Router::new()
        .route("/get", get(handlers::get_blocktypes))
        .route("/new", post(handlers::new_blocktype));

    // Timeblock-related routes
    let timeblock_routes = Router::new()
        .route("/get", get(handlers::get_daydata))
        .route("/next", post(handlers::next_timeblock))
        .route("/split", post(handlers::split_timeblock))
        .route("/adjust", post(handlers::adjust_timeblock));

    // Current block-related routes
    let currentblock_routes = Router::new()
        .route("/change", post(handlers::change_current_block))
        .route("/get", get(handlers::get_current_block));

    // NOTE: User management routes
    let auth_routes = Router::new()
        .route("/handshake", get(auth::handlers::handshake))
        .route("/device", post(auth::handlers::register))
        .route("/info", post(auth::handlers::get_access_token));

    let security_route = Router::new().nest("/user", auth_routes);

    // Main application routes
    let routes = Router::new()
        .route("/state", get(handlers::get_entire_state)) // Main state route
        .nest("/blocktype", blocktype_routes) // Grouped blocktype routes
        .nest("/timeblock", timeblock_routes) // Grouped timeblock routes
        .nest("/currentblock", currentblock_routes) // Grouped current block routes
        .route("/analysis", get(handlers::get_analysis)) // Analysis route
        .layer(from_fn_with_state(
            data.clone(),
            auth::middleware::auth_middleware,
        ))
        .nest("/security", security_route)
        .with_state(data);

    let router_service = routes.into_make_service();
    let listener = TcpListener::bind(&ip).await?;
    serve(listener, router_service).await?;
    Ok(())
}
