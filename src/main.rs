use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::{Response, StatusCode},
    routing::get,
    Json, Router, TypedHeader,
};
use axum_macros::debug_handler;
use sha256::digest;
use std::{env, path::Path};

pub mod blocktype;
pub mod duration;
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
        .route("/blocktypes", get(get_blocktypes).post(post_blocktypes))
        .route("/timeblocks", get(get_timeblocks).post(post_timeblocks))
        .route(
            "/currentblockname",
            get(get_currentblockname).post(post_currentblockname),
        )
        .route(
            "/currentblocktype",
            get(get_currentblocktype).post(post_currentblocktype),
        );
    let router_service = routes.with_state(state).into_make_service();
    axum::Server::bind(&ip.parse()?)
        .serve(router_service)
        .await?;
    Ok(())
}

async fn get_blocktypes(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Response<String> {
    let password_hash = state.password_hash.clone();
    if auth_header.token() != password_hash {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("".to_string())
            .unwrap()
    } else {
        let blocktypes = blocktype::BlockType::load();
        if let Ok(blocktypes) = blocktypes {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&blocktypes).unwrap())
                .unwrap()
        } else {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("".to_string())
                .unwrap()
        }
    }
}

async fn post_blocktypes(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    body: Json<String>,
) -> Response<String> {
    let password_hash = state.password_hash.clone();
    if auth_header.token() != password_hash {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("".to_string())
            .unwrap()
    } else {
        let blocktype = serde_json::from_str::<blocktype::BlockType>(&body.0);
        if let Ok(blocktype) = blocktype {
            let blocktypes = blocktype::BlockType::load();
            if let Ok(mut blocktypes) = blocktypes {
                blocktypes.push(blocktype);
                let result = blocktype::BlockType::save();
                if result.is_ok() {
                    Response::builder()
                        .status(StatusCode::OK)
                        .body("".to_string())
                        .unwrap()
                } else {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("".to_string())
                        .unwrap()
                }
            } else {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("".to_string())
                    .unwrap()
            }
        } else {
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("".to_string())
                .unwrap()
        }
    }
}

async fn get_timeblocks() -> String {
    unimplemented!()
}

async fn post_timeblocks() -> String {
    unimplemented!()
}

#[debug_handler]
async fn get_currentblockname(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Response<String> {
    let password_hash = state.password_hash.clone();
    todo!()
}

async fn post_currentblockname() -> String {
    unimplemented!()
}

async fn get_currentblocktype() -> String {
    unimplemented!()
}

async fn post_currentblocktype() -> String {
    unimplemented!()
}
