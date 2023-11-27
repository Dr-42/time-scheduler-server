use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::{Response, StatusCode},
    Json, TypedHeader,
};

use crate::blocktype;
use crate::AppState;
use axum_macros::debug_handler;

pub async fn get_blocktypes(
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

pub async fn post_blocktypes(
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

pub async fn get_timeblocks() -> String {
    unimplemented!()
}

pub async fn post_timeblocks() -> String {
    unimplemented!()
}

#[debug_handler]
pub async fn get_currentblockname(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Response<String> {
    let password_hash = state.password_hash.clone();
    todo!()
}

pub async fn post_currentblockname() -> String {
    unimplemented!()
}

pub async fn get_currentblocktype() -> String {
    unimplemented!()
}

pub async fn post_currentblocktype() -> String {
    unimplemented!()
}
