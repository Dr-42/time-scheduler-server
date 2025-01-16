#![deny(clippy::unwrap_used, clippy::expect_used)]

use app::AppState;
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    serve, Router,
};
use sha256::digest;
use std::{env, path::Path};
use tokio::net::TcpListener;

mod analysis;
mod app;
mod blocktype;
mod currentblock;
mod err;
mod handlers;
mod middleware;
mod migrator;
mod timeblock;

macro_rules! password_input {
    ($($fmt:expr),*) => {
        {
            use std::io::{self, Write};
            print!($($fmt),*);
            io::stdout().flush().unwrap();
            let input = rpassword::read_password()?;
            input
        }
    };
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();
    let mut port = 8080;
    if let Some(subcmd) = args.get(1) {
        if subcmd == "--help" {
            println!("Usage: {} <port>", args[0]);
            return Ok(());
        } else if subcmd == "migrate" {
            let overwrite = args.get(2).map(|s| s == "--overwrite").unwrap_or(false);
            migrator::migrate(overwrite).await;
            return Ok(());
        }
    }
    if args.len() == 2 {
        port = args[1].parse::<u16>()?;
    } else {
        println!("Usage: {} <port>", args[0]);
    }

    let ip = format!("0.0.0.0:{}", port);

    if !Path::new("password.txt").exists() {
        let password = password_input!("Enter a password: ");
        let hash = digest(password);
        std::fs::write("password.txt", hash)?;
    }
    let password_hash = std::fs::read_to_string("password.txt")?;
    println!("Password hash: {}", password_hash);

    let state = AppState::init(password_hash).await;

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

    // Main application routes
    let routes = Router::new()
        .route("/state", get(handlers::get_entire_state)) // Main state route
        .nest("/blocktype", blocktype_routes) // Grouped blocktype routes
        .nest("/timeblock", timeblock_routes) // Grouped timeblock routes
        .nest("/currentblock", currentblock_routes) // Grouped current block routes
        .route("/analysis", get(handlers::get_analysis)) // Analysis route
        .layer(from_fn_with_state(state, middleware::auth_middleware));

    let router_service = routes.into_make_service();
    let listener = TcpListener::bind(&ip).await?;
    serve(listener, router_service).await?;
    Ok(())
}
