use crate::{blocktype, time::Time, timeblock, AppState};
use axum::{
    extract::{Query, State},
    headers::{authorization::Bearer, Authorization},
    http::{Response, StatusCode},
    Json, TypedHeader,
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize)]
pub struct GetTimeblocksQuery {
    year: u32,
    month: u8,
    day: u8,
}

#[debug_handler]
pub async fn get_timeblocks(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    query: Query<GetTimeblocksQuery>,
) -> Response<String> {
    let password_hash = state.password_hash.clone();
    if auth_header.token() != password_hash {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("".to_string())
            .unwrap()
    } else {
        let req_date = Time::new(query.year, query.month, query.day, 0, 0, 0);
        if let Err(e) = req_date {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(e.to_string())
                .unwrap();
        }
        let req_date = req_date.unwrap();
        let timeblocks = timeblock::TimeBlock::get_day_timeblocks(&req_date);
        if timeblocks.is_ok() {
            let timeblocks = timeblocks.unwrap();
            let time_string = serde_json::to_string(&req_date);
            if time_string.is_ok() {
                let timeblocks = serde_json::to_string(&timeblocks);
                if let Ok(timeblocks) = timeblocks {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .body(timeblocks)
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
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("".to_string())
                .unwrap()
        }
    }
}

pub async fn post_timeblocks(
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
        unimplemented!()
    }
}

#[debug_handler]
pub async fn get_currentblockname(
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
        unimplemented!()
    }
}

pub async fn post_currentblockname(
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
        unimplemented!()
    }
}

pub async fn get_currentblocktype(
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
        unimplemented!()
    }
}

pub async fn post_currentblocktype(
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
        unimplemented!()
    }
}

pub async fn get_analysis(
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
        unimplemented!()
    }
}
