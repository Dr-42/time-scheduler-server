use axum::{
    body::Body,
    extract::Query,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Local, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::{
    analysis::{Analysis, AnalysisQuery},
    blocktype::{BlockType, NewBlockType, PushNew},
    currentblock::CurrentBlock,
    err::{Error, ErrorType},
    err_from_type, err_with_context,
    timeblock::{SplitTimeBlockQuery, TimeBlock},
};

#[derive(Serialize, Deserialize)]
pub struct EntireState {
    blocktypes: Vec<BlockType>,
    daydata: Vec<TimeBlock>,
    currentblock: CurrentBlock,
}

pub async fn get_entire_state() -> Result<impl IntoResponse, Error> {
    println!("Getting home state for today");
    let blocktypes = BlockType::load().await?;
    let daydata = TimeBlock::get_day_timeblocks(Local::now().date_naive()).await?;
    let currentblock = CurrentBlock::get().await?;
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

pub async fn get_blocktypes() -> Result<impl IntoResponse, Error> {
    println!("Getting block types");
    let blocktypes = BlockType::load().await?;
    let response_body = serde_json::to_string(&blocktypes)
        .map_err(|e| err_with_context!(e, "Serializing block types"))?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))
        .map_err(|e| err_with_context!(e, "Building response for block types"))
}

pub async fn new_blocktype(
    Json(blocktype): Json<NewBlockType>,
) -> Result<impl IntoResponse, Error> {
    match BlockType::load().await {
        Ok(mut current_blocks) => {
            println!("Saving new block type {:?}", &blocktype);
            current_blocks.push_new(blocktype);
            BlockType::save(&current_blocks).await?;
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

pub async fn get_daydata(Query(day): Query<DayDataQuery>) -> Result<impl IntoResponse, Error> {
    println!("Getting day data for {:?}", day.date);
    let timeblocks = TimeBlock::get_day_timeblocks(day.date.date_naive()).await?;
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
    Json(new_current_block): Json<CurrentBlock>,
) -> Result<impl IntoResponse, Error> {
    let time_blocks = TimeBlock::get_day_timeblocks(Local::now().date_naive()).await?;
    let current_data = CurrentBlock::get().await?;
    let time_blocks = if time_blocks.is_empty() {
        // Get previous day
        TimeBlock::get_day_timeblocks(Local::now().date_naive() - chrono::Duration::days(1)).await?
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
    timeblock.save().await?;
    new_current_block.save().await?;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Time block saved"))
        .map_err(|e| err_with_context!(e, "Building response next timeblock"))
}

pub async fn split_timeblock(
    Json(split_time_block_query): Json<SplitTimeBlockQuery>,
) -> Result<impl IntoResponse, Error> {
    println!("Splitting timeblock for {:?}", split_time_block_query);
    TimeBlock::split_timeblock(split_time_block_query).await?;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Time block split"))
        .map_err(|e| err_with_context!(e, "Building response split timeblock"))
}

pub async fn change_current_block(
    Json(current_block): Json<CurrentBlock>,
) -> Result<impl IntoResponse, Error> {
    println!("Changing current block to {:?}", current_block);
    current_block.save().await?;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Current block saved"))
        .map_err(|e| err_with_context!(e, "Building response change current block"))
}

pub async fn get_current_block() -> Result<impl IntoResponse, Error> {
    println!("Getting current data");
    let current_block = CurrentBlock::get().await?;
    let response_body = serde_json::to_string(&current_block)
        .map_err(|e| err_with_context!(e, "Serializing current block"))?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))
        .map_err(|e| err_with_context!(e, "Building response for current block"))
}

pub async fn get_analysis(Query(query): Query<AnalysisQuery>) -> Result<impl IntoResponse, Error> {
    println!(
        "Getting analysis data from {:?} to {:?}",
        query.start, query.end
    );
    let analysis = Analysis::get_analysis_data(query.start, query.end).await?;
    let response_body = serde_json::to_string(&analysis)
        .map_err(|e| err_with_context!(e, "Serializing analysis data"))?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))
        .map_err(|e| err_with_context!(e, "Building response for analysis data"))
}
