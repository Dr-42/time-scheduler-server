use std::collections::HashMap;

use axum::{
    body::Body,
    extract::{Query, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::{
    analysis::{Analysis, AnalysisQuery},
    app::AppData,
    blocktype::{BlockType, NewBlockType, PushNew},
    currentblock::CurrentBlock,
    err_from_type, err_with_context,
    error::{Error, ErrorType},
    sync::Sync,
    timeblock::{AdjustTimeBlockQuery, SplitTimeBlockQuery, TimeBlock},
};

#[derive(Serialize, Deserialize)]
pub struct EntireState {
    blocktypes: Vec<BlockType>,
    daydata: Vec<TimeBlock>,
    currentblock: CurrentBlock,
}

pub async fn get_entire_state(State(data): State<AppData>) -> Result<impl IntoResponse, Error> {
    println!("Getting home state for today");
    let blocktypes = BlockType::load(&data.data_dir).await?;
    let daydata = TimeBlock::get_day_timeblocks(&data.data_dir, Local::now().date_naive()).await?;
    let currentblock = CurrentBlock::get(&data.data_dir).await?;
    let entire_state = EntireState {
        blocktypes,
        daydata,
        currentblock,
    };

    let response_body = serde_json::to_string(&entire_state)
        .map_err(|e| err_with_context!(e, "Serializing entire state"))?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))
        .map_err(|e| err_with_context!(e, "Building response for entire state"))
}

pub async fn get_blocktypes(State(data): State<AppData>) -> Result<impl IntoResponse, Error> {
    println!("Getting block types");
    let blocktypes = BlockType::load(&data.data_dir).await?;
    let response_body = serde_json::to_string(&blocktypes)
        .map_err(|e| err_with_context!(e, "Serializing block types"))?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))
        .map_err(|e| err_with_context!(e, "Building response for block types"))
}

pub async fn new_blocktype(
    State(data): State<AppData>,
    Json(blocktype): Json<NewBlockType>,
) -> Result<impl IntoResponse, Error> {
    match BlockType::load(&data.data_dir).await {
        Ok(mut current_blocks) => {
            println!("Saving new block type {:?}", &blocktype);
            current_blocks.push_new(blocktype);
            BlockType::save(&data.data_dir, &current_blocks).await?;
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from("Block type saved"))
                .map_err(|e| err_with_context!(e, "Building response for block types"))?)
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .map_err(|e| err_with_context!(e, "Building error response for block types")),
    }
}

#[derive(Serialize, Deserialize)]
pub struct DayDataQuery {
    date: DateTime<Local>,
}

pub async fn get_daydata(
    State(data): State<AppData>,
    Query(day): Query<DayDataQuery>,
) -> Result<impl IntoResponse, Error> {
    println!("Getting day data for {:?}", day.date);
    let timeblocks = TimeBlock::get_day_timeblocks(&data.data_dir, day.date.date_naive()).await?;
    let response_body = serde_json::to_string(&timeblocks).map_err(|e| {
        err_with_context!(
            e,
            "Serializing timeblocks for {}",
            day.date.format("%Y-%m-%d")
        )
    })?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))
        .map_err(|e| {
            err_with_context!(
                e,
                "Building response for timeblocks for {}",
                day.date.format("%Y-%m-%d")
            )
        })
}

pub async fn next_timeblock(
    State(data): State<AppData>,
    Json(new_current_block): Json<CurrentBlock>,
) -> Result<impl IntoResponse, Error> {
    let time_blocks =
        TimeBlock::get_day_timeblocks(&data.data_dir, Local::now().date_naive()).await?;
    let current_data = CurrentBlock::get(&data.data_dir).await?;
    let time_blocks = if time_blocks.is_empty() {
        // Get previous day
        TimeBlock::get_day_timeblocks(
            &data.data_dir,
            Local::now().date_naive() - chrono::Duration::days(1),
        )
        .await?
    } else {
        time_blocks
    };
    let start_time = if time_blocks.is_empty() {
        Local::now()
            .with_time(NaiveTime::from_hms_opt(0, 0, 0).ok_or(err_from_type!(
                ErrorType::Chrono,
                "Creating start time for {}",
                Local::now().format("%Y-%m-%d")
            ))?)
            .single()
            .ok_or(err_from_type!(
                ErrorType::Chrono,
                "No single time identifiable for start time for {}",
                Local::now().format("%Y-%m-%d")
            ))?
    } else {
        time_blocks
            .last()
            .ok_or(err_from_type!(
                ErrorType::InternalRustError,
                "This should never happen"
            ))?
            .end_time
    };
    let end_time = Local::now();
    let timeblock = TimeBlock::new(
        start_time,
        end_time,
        current_data.block_type_id,
        current_data.current_block_name,
    );
    timeblock.save(&data.data_dir).await?;
    new_current_block.save(&data.data_dir).await?;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Time block saved"))
        .map_err(|e| err_with_context!(e, "Building response next timeblock"))
}

pub async fn split_timeblock(
    State(data): State<AppData>,
    Json(split_time_block_query): Json<SplitTimeBlockQuery>,
) -> Result<impl IntoResponse, Error> {
    println!("Splitting timeblock for {:?}", split_time_block_query);
    TimeBlock::split_timeblock(&data.data_dir, split_time_block_query).await?;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Time block split"))
        .map_err(|e| err_with_context!(e, "Building response split timeblock"))
}

pub async fn adjust_timeblock(
    State(data): State<AppData>,
    Json(adjust_time_block_query): Json<AdjustTimeBlockQuery>,
) -> Result<impl IntoResponse, Error> {
    println!("Adjusting timeblock for {:?}", adjust_time_block_query);
    TimeBlock::adjust_timeblock(&data.data_dir, adjust_time_block_query).await?;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Time block adjusted"))
        .map_err(|e| err_with_context!(e, "Building response adjust timeblock"))
}

pub async fn change_current_block(
    State(data): State<AppData>,
    Json(current_block): Json<CurrentBlock>,
) -> Result<impl IntoResponse, Error> {
    println!("Changing current block to {:?}", current_block);
    current_block.save(&data.data_dir).await?;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Current block saved"))
        .map_err(|e| err_with_context!(e, "Building response change current block"))
}

pub async fn get_current_block(State(data): State<AppData>) -> Result<impl IntoResponse, Error> {
    println!("Getting current data");
    let current_block = CurrentBlock::get(&data.data_dir).await?;
    let response_body = serde_json::to_string(&current_block)
        .map_err(|e| err_with_context!(e, "Serializing current block"))?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))
        .map_err(|e| err_with_context!(e, "Building response for current block"))
}

pub async fn get_analysis(
    State(data): State<AppData>,
    Query(query): Query<AnalysisQuery>,
) -> Result<impl IntoResponse, Error> {
    println!(
        "Getting analysis data from {:?} to {:?}",
        query.start, query.end
    );
    let analysis = Analysis::get_analysis_data(&data.data_dir, query.start, query.end).await?;
    let response_body = serde_json::to_string(&analysis)
        .map_err(|e| err_with_context!(e, "Serializing analysis data"))?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))
        .map_err(|e| err_with_context!(e, "Building response for analysis data"))
}

pub async fn get_last_update(State(data): State<AppData>) -> Result<impl IntoResponse, Error> {
    println!("Getting last update");
    let last_update_path = data.data_dir.join("last_update.txt");
    let last_update = tokio::fs::read_to_string(&last_update_path)
        .await
        .map_err(|e| err_with_context!(e, "Reading last_update.txt"))?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(last_update))
        .map_err(|e| err_with_context!(e, "Building response for last update"))
}

#[derive(Deserialize, Serialize)]
pub struct SyncBody {
    time_blocks: HashMap<NaiveDate, Vec<TimeBlock>>,
    current_block: CurrentBlock,
    block_types: Vec<BlockType>,
}

pub async fn sync(
    State(app_data): State<AppData>,
    Json(data): Json<SyncBody>,
) -> Result<impl IntoResponse, Error> {
    println!("Syncing data");
    BlockType::sync(&app_data.data_dir, data.block_types).await?;
    TimeBlock::sync(&app_data.data_dir, data.time_blocks).await?;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Synced"))
        .map_err(|e| err_with_context!(e, "Building response for syncing data"))
}
