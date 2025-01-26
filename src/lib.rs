#![deny(clippy::unwrap_used, clippy::expect_used)]

use app::{AppData, AppState};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Extension, Router,
};
use sha256::digest;
use std::path::PathBuf;
use tokio::{net::TcpListener, task::JoinHandle};

mod analysis;
mod app;
mod auth;
mod blocktype;
mod currentblock;
pub mod error;
mod handlers;
mod sync;
mod timeblock;

use error::{Error, ErrorType};

pub struct App {
    port: u16,
    data_dir: PathBuf,
    run_handle: Option<JoinHandle<Result<(), Error>>>,
}

impl App {
    pub fn new(port: u16, data_dir: PathBuf) -> Self {
        Self {
            port,
            data_dir,
            run_handle: None,
        }
    }

    pub async fn set_password(&mut self, password: String) -> Result<(), Error> {
        let password_path = self.data_dir.join("password.txt");
        let password_hash = digest(&password);
        std::fs::write(&password_path, password_hash).map_err(|e| {
            err_with_context!(
                e,
                "Failed to write password file {}",
                password_path.display()
            )
        })?;
        Ok(())
    }

    pub async fn init(&mut self) -> Result<(), Error> {
        println!("Starting server on port {}", self.port);
        let ip = format!("0.0.0.0:{}", self.port);
        let password_path = self.data_dir.join("password.txt");
        let password_hash = std::fs::read_to_string(&password_path).map_err(|e| {
            err_with_context!(
                e,
                "Failed to read password file {}",
                password_path.display()
            )
        })?;
        let state = AppState::init(password_hash).await;
        let app_data = AppData::init(self.data_dir.clone()).await;

        let routes = Router::new()
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
            // Last updated on
            .route("/lastupdated", get(handlers::get_last_update))
            // Sync
            .route("/sync", post(handlers::sync))
            .layer(from_fn_with_state(
                state.clone(),
                auth::middleware::auth_middleware,
            ))
            // Auth
            .route("/auth/login", post(auth::handlers::login))
            .route("/auth/refresh", post(auth::handlers::refresh_token))
            .route("/auth/check", post(auth::handlers::check_token))
            .layer(Extension(state.clone()))
            .with_state(app_data);

        let listener = TcpListener::bind(&ip)
            .await
            .map_err(|e| err_with_context!(e, "Failed to bind"))?;
        self.run_handle = Some(tokio::spawn(async move {
            println!("Listening on {}", ip);
            axum::serve(listener, routes)
                .await
                .map_err(|e| err_with_context!(e, "Failed to start server"))
        }));
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        if let Some(handle) = self.run_handle.take() {
            handle
                .await
                .map_err(|e| err_with_context!(e, "Failed to run server"))?
        } else {
            Err(err_from_type!(
                ErrorType::NotInitialized,
                "Please run the init method before run"
            ))
        }
    }

    pub async fn stop(&mut self) {
        if let Some(handle) = self.run_handle.take() {
            handle.abort();
        }
    }
}
