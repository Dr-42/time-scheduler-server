use std::fmt::Display;

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
    analysis::{Analysis, AnalysisError, AnalysisQuery},
    blocktype::{BlockType, BlockTypeError, NewBlockType, PushNew},
    currentblock::{CurrentBlock, CurrentBlockError},
    timeblock::{TimeBlock, TimeBlockError},
};

pub enum HandlerError {
    AxumError(axum::http::Error),
    SerdeError(serde_json::Error),
    Tokio(tokio::io::Error),
    Chrono,
    IdenticalBlockType,
    InternalRustError,
}

impl From<axum::http::Error> for HandlerError {
    fn from(e: axum::http::Error) -> Self {
        HandlerError::AxumError(e)
    }
}

impl From<serde_json::Error> for HandlerError {
    fn from(e: serde_json::Error) -> Self {
        HandlerError::SerdeError(e)
    }
}

impl From<TimeBlockError> for HandlerError {
    fn from(e: TimeBlockError) -> Self {
        match e {
            TimeBlockError::Serde(error) => HandlerError::SerdeError(error),
            TimeBlockError::Tokio(error) => HandlerError::Tokio(error),
            TimeBlockError::Chrono => HandlerError::Chrono,
        }
    }
}

impl From<CurrentBlockError> for HandlerError {
    fn from(e: CurrentBlockError) -> Self {
        match e {
            CurrentBlockError::Tokio(error) => HandlerError::Tokio(error),
            CurrentBlockError::Serde(error) => HandlerError::SerdeError(error),
        }
    }
}

impl From<BlockTypeError> for HandlerError {
    fn from(e: BlockTypeError) -> Self {
        match e {
            BlockTypeError::Identical => HandlerError::IdenticalBlockType,
            BlockTypeError::Tokio(error) => HandlerError::Tokio(error),
            BlockTypeError::Serde(error) => HandlerError::SerdeError(error),
        }
    }
}

impl From<AnalysisError> for HandlerError {
    fn from(e: AnalysisError) -> Self {
        match e {
            AnalysisError::Tokio(error) => HandlerError::Tokio(error),
            AnalysisError::Serde(error) => HandlerError::SerdeError(error),
            AnalysisError::Chrono => HandlerError::Chrono,
            AnalysisError::BlockTypeIdentical => HandlerError::IdenticalBlockType,
        }
    }
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response<Body> {
        // Here if I can't create the body then fuck me
        #[allow(clippy::unwrap_used)]
        fn create_error_response<T: Display>(message: T) -> Response<Body> {
            let status_code = StatusCode::INTERNAL_SERVER_ERROR;
            Response::builder()
                .status(status_code)
                .body(Body::from(message.to_string()))
                .unwrap()
        }
        match self {
            HandlerError::AxumError(e) => create_error_response(e),
            HandlerError::SerdeError(e) => create_error_response(e),
            HandlerError::Tokio(e) => create_error_response(e),
            HandlerError::Chrono => create_error_response("Chrono error"),
            HandlerError::IdenticalBlockType => create_error_response("Identical block type"),
            HandlerError::InternalRustError => create_error_response("Internal rust error"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EntireState {
    blocktypes: Vec<BlockType>,
    daydata: Vec<TimeBlock>,
    currentblock: CurrentBlock,
}

pub async fn get_entire_state() -> Result<impl IntoResponse, HandlerError> {
    println!("Getting home state for today");
    let blocktypes = BlockType::load().await?;
    let daydata = TimeBlock::get_day_timeblocks(Local::now().date_naive()).await?;
    let currentblock = CurrentBlock::get().await?;
    let entire_state = EntireState {
        blocktypes,
        daydata,
        currentblock,
    };

    let response_body = serde_json::to_string(&entire_state)?;
    let status_code = StatusCode::OK;
    Ok(Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))?)
}

pub async fn get_blocktypes() -> Result<impl IntoResponse, HandlerError> {
    println!("Getting block types");
    let blocktypes = BlockType::load().await?;
    let response_body = serde_json::to_string(&blocktypes)?;
    let status_code = StatusCode::OK;
    Ok(Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))?)
}

pub async fn new_blocktype(
    Json(blocktype): Json<NewBlockType>,
) -> Result<impl IntoResponse, HandlerError> {
    let mut current_blocks = match BlockType::load().await {
        Ok(current_blocks) => current_blocks,
        Err(e) => {
            let status_code = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(Response::builder()
                .status(status_code)
                .body(Body::from(e.to_string()))?);
        }
    };
    println!("Saving new block type {:?}", &blocktype);
    current_blocks.push_new(blocktype);
    BlockType::save(current_blocks).await?;
    let status_code = StatusCode::OK;
    Ok(Response::builder()
        .status(status_code)
        .body(Body::from("Block type saved"))?)
}

#[derive(Serialize, Deserialize)]
pub struct DayDataQuery {
    date: DateTime<Local>,
}

pub async fn get_daydata(
    Query(day): Query<DayDataQuery>,
) -> Result<impl IntoResponse, HandlerError> {
    println!("Getting day data for {:?}", day.date);
    let timeblocks = TimeBlock::get_day_timeblocks(day.date.date_naive()).await?;
    let response_body = serde_json::to_string(&timeblocks)?;
    let status_code = StatusCode::OK;
    Ok(Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))?)
}

pub async fn next_timeblock(
    Json(new_current_block): Json<CurrentBlock>,
) -> Result<impl IntoResponse, HandlerError> {
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
            .with_time(NaiveTime::from_hms_opt(0, 0, 0).ok_or(HandlerError::Chrono)?)
            .single()
            .ok_or(HandlerError::Chrono)?
    } else {
        time_blocks
            .last()
            .ok_or(HandlerError::InternalRustError)?
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

    let status_code = StatusCode::OK;
    Ok(Response::builder()
        .status(status_code)
        .body(Body::from("Time block saved"))?)
}

pub async fn change_current_block(
    Json(current_block): Json<CurrentBlock>,
) -> Result<impl IntoResponse, HandlerError> {
    println!("Changing current block to {:?}", current_block);
    current_block.save().await?;
    let status_code = StatusCode::OK;
    Ok(Response::builder()
        .status(status_code)
        .body(Body::from("Current block saved"))?)
}

pub async fn get_current_data() -> Result<impl IntoResponse, HandlerError> {
    println!("Getting current data");
    let current_block = CurrentBlock::get().await?;
    let response_body = serde_json::to_string(&current_block)?;
    let status_code = StatusCode::OK;
    Ok(Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))?)
}

pub async fn get_analysis(
    Query(query): Query<AnalysisQuery>,
) -> Result<impl IntoResponse, HandlerError> {
    println!(
        "Getting analysis data from {:?} to {:?}",
        query.start, query.end
    );
    let analysis = Analysis::get_analysis_data(query.start, query.end).await?;
    let response_body = serde_json::to_string(&analysis)?;
    let status_code = StatusCode::OK;
    Ok(Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(response_body))?)
}
