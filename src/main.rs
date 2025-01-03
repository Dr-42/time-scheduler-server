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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let routes = Router::new()
        // Sends all the data for today. Will be most used
        .route("/state", get(handlers::get_entire_state))
        // Gets the blocktypes
        .route("/blocktypes", get(handlers::get_blocktypes))
        // New Blocktype
        .route("/newblocktype", post(handlers::new_blocktype))
        // Gets the timeblocks for a spqcified date
        .route("/daydata", get(handlers::get_daydata))
        // Posts a new timeblock, ie user starts a new task
        .route("/nexttimeblock", post(handlers::next_timeblock))
        // Posts a change to the current timeblock
        .route("/changecurrentblock", post(handlers::change_current_block))
        // Get the current data
        .route("/getcurrentdata", get(handlers::get_current_data))
        // Get analysis for a given date range
        .route("/analysis", get(handlers::get_analysis))
        // Check authorization
        .layer(from_fn_with_state(state, middleware::auth_middleware));
    let router_service = routes.into_make_service();
    let listener = TcpListener::bind(&ip).await?;
    serve(listener, router_service).await?;
    Ok(())
}
