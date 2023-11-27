use axum::{routing::get, Router};
use sha256::digest;
use std::{env, path::Path};

pub mod blocktype;
pub mod duration;
pub mod handlers;
pub mod time;
pub mod timeblock;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

macro_rules! input {
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

#[derive(Debug, Clone)]
struct AppState {
    password_hash: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let mut port = 8080;
    if args.len() == 2 {
        port = args[1].parse::<u16>()?;
    } else {
        println!("Usage: {} <port>", args[0]);
    }

    let ip = format!("127.0.0.1:{}", port);

    if !Path::new("password.txt").exists() {
        let password = input!("Enter a password: ");
        let hash = digest(password);
        std::fs::write("password.txt", hash)?;
    }
    let password_hash = std::fs::read_to_string("password.txt")?;
    println!("Password hash: {}", password_hash);

    let state = AppState { password_hash };
    let routes = Router::new()
        .route(
            "/blocktypes",
            get(handlers::get_blocktypes).post(handlers::post_blocktypes),
        )
        .route(
            "/timeblocks",
            get(handlers::get_timeblocks).post(handlers::post_timeblocks),
        )
        .route(
            "/currentblockname",
            get(handlers::get_currentblockname).post(handlers::post_currentblockname),
        )
        .route(
            "/currentblocktype",
            get(handlers::get_currentblocktype).post(handlers::post_currentblocktype),
        );
    let router_service = routes.with_state(state).into_make_service();
    axum::Server::bind(&ip.parse()?)
        .serve(router_service)
        .await?;
    Ok(())
}
