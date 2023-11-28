use crate::{
    blocktype,
    duration::Duration,
    time::Time,
    timeblock::{self, TimeBlock},
    AppState, Result,
};
use axum::{
    extract::{Query, State},
    headers::{authorization::Bearer, Authorization},
    http::{Response, StatusCode},
    Json, TypedHeader,
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
                if result.is_err() {
                    println!("Error saving blocktypes: {}", result.unwrap_err());
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("".to_string())
                        .unwrap()
                } else {
                    Response::builder()
                        .status(StatusCode::OK)
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
    body: Json<TimeBlock>,
) -> Response<String> {
    let password_hash = state.password_hash.clone();
    if auth_header.token() != password_hash {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("".to_string())
            .unwrap()
    } else {
        let mut timeblock = body.0;
        let result = timeblock.save();
        if let Err(e) = result {
            println!("Error saving timeblock: {}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("".to_string())
                .unwrap()
        } else {
            Response::builder()
                .status(StatusCode::OK)
                .body("".to_string())
                .unwrap()
        }
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
        if !std::path::Path::new("currentblockname.txt").exists() {
            std::fs::File::create("currentblockname.txt").unwrap();
            std::fs::write("currentblockname.txt", "Setting Up for first use").unwrap();
        }
        let current_block_name = std::fs::read_to_string("currentblockname.txt");
        if let Ok(current_block_name) = current_block_name {
            let current_block_name = current_block_name.trim().to_string();
            let current_block_name = format!("\"{}\"", current_block_name);
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(current_block_name)
                .unwrap()
        } else {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("".to_string())
                .unwrap()
        }
    }
}

pub async fn post_currentblockname(
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
        let result = std::fs::write("currentblockname.txt", body.0);
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
        if !std::path::Path::new("currentblocktype.txt").exists() {
            std::fs::File::create("currentblocktype.txt").unwrap();
            std::fs::write("currentblocktype.txt", "0").unwrap();
        }
        let current_block_type = std::fs::read_to_string("currentblocktype.txt");
        if let Ok(current_block_type) = current_block_type {
            let current_block_type = current_block_type.trim().to_string();
            let current_block_type = format!("\"{}\"", current_block_type);
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(serde_json::from_str(&current_block_type).unwrap())
                .unwrap()
        } else {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("".to_string())
                .unwrap()
        }
    }
}

pub async fn post_currentblocktype(
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
        let result = std::fs::write("currentblocktype.txt", body.0);
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
    }
}

#[derive(Serialize, Deserialize)]
pub struct AnalysisQuery {
    pub startday: u8,
    pub startmonth: u8,
    pub startyear: u32,
    pub endday: u8,
    pub endmonth: u8,
    pub endyear: u32,
}

#[debug_handler]
pub async fn get_analysis(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    query: Query<AnalysisQuery>,
) -> Response<String> {
    let password_hash = state.password_hash.clone();
    if auth_header.token() != password_hash {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("".to_string())
            .unwrap()
    } else {
        let start_time = Time::new(query.startyear, query.startmonth, query.startday, 0, 0, 0);
        let end_time = Time::new(query.endyear, query.endmonth, query.endday, 23, 59, 59);
        if let Err(e) = start_time {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(e.to_string())
                .unwrap();
        }
        if let Err(e) = end_time {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(e.to_string())
                .unwrap();
        }

        let start_time = start_time.unwrap();
        let end_time = end_time.unwrap();
        let analysis = get_analysis_data(start_time, end_time);
        if let Err(e) = analysis {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(e.to_string())
                .unwrap();
        }
        let analysis = serde_json::to_string(&analysis.unwrap());
        if let Ok(analysis) = analysis {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(analysis)
                .unwrap()
        } else {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("".to_string())
                .unwrap()
        }
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Trend {
    pub day: Time,
    pub timeSpent: Duration,
    pub blockTypeID: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Analysis {
    pub percentages: Vec<f32>,
    pub trends: Vec<Trend>,
}

fn get_analysis_data(start_time: Time, end_time: Time) -> Result<Analysis> {
    let mut blocktypes = blocktype::BlockType::load()?;
    blocktypes.sort_by(|a, b| a.id.cmp(&b.id));

    let mut iter_time = start_time;
    let mut durations: HashMap<u8, Duration> = HashMap::new();
    let mut trends: Vec<Trend> = Vec::new();

    while iter_time.before(&end_time) {
        let blocks = timeblock::TimeBlock::get_day_timeblocks(&iter_time)?;
        for blocktype in &blocktypes {
            let mut time_spent = Duration {
                seconds: 0,
                minutes: 0,
                hours: 0,
            };
            for block in &blocks {
                if block.blockTypeId == blocktype.id {
                    time_spent += block.duration();
                }
            }

            let trend = Trend {
                day: iter_time,
                timeSpent: time_spent,
                blockTypeID: blocktype.id,
            };
            trends.push(trend);
            if durations.contains_key(&blocktype.id) {
                durations.insert(blocktype.id, durations[&blocktype.id] + time_spent);
            } else {
                durations.insert(blocktype.id, time_spent);
            }
        }

        iter_time += Duration::from_seconds(24 * 60 * 60);
    }

    let mut total_time = Duration {
        seconds: 0,
        minutes: 0,
        hours: 0,
    };
    for duration in durations.values() {
        total_time += *duration;
    }

    let mut percentage_map: HashMap<u8, f32> = HashMap::new();
    for (blocktype_id, duration) in &durations {
        let percentage = (duration.to_seconds() as f32) / (total_time.to_seconds() as f32);
        percentage_map.insert(*blocktype_id, percentage);
    }

    let mut percentages: Vec<f32> = Vec::new();
    percentages.resize(blocktypes.len(), 0.0);
    for (blocktype_id, percentage) in &percentage_map {
        percentages[*blocktype_id as usize] = *percentage;
    }
    Ok(Analysis {
        percentages,
        trends,
    })
}
