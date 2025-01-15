use std::path::Path;

use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::{
    err::{Error, ErrorType},
    err_from_type, err_with_context,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TimeBlock {
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub block_type_id: u8,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SplitTimeBlockQuery {
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    split_time: DateTime<Local>,
    before_title: String,
    after_title: String,
    before_block_type_id: u8,
    after_block_type_id: u8,
}

impl TimeBlock {
    pub fn new(
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
        block_type_id: u8,
        title: String,
    ) -> TimeBlock {
        TimeBlock {
            start_time,
            end_time,
            block_type_id,
            title,
        }
    }

    pub fn duration(&self) -> chrono::Duration {
        self.end_time - self.start_time
    }

    pub async fn get_day_timeblocks(day: NaiveDate) -> Result<Vec<TimeBlock>, Error> {
        if !Path::new("timeblocks").exists() {
            tokio::fs::create_dir("timeblocks")
                .await
                .map_err(|e| err_with_context!(e, "Creating timeblocks directory"))?;
        }

        let file_name = format!("timeblocks/{}.json", day.format("%Y-%m-%d"));
        let file = tokio::fs::File::open(file_name).await;
        if let Err(e) = &file {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok(vec![]);
            }
        }
        let mut file = file.map_err(|e| {
            err_with_context!(e, "Opening timeblocks/{}.json", day.format("%Y-%m-%d"))
        })?;
        let mut content = String::new();
        tokio::io::AsyncReadExt::read_to_string(&mut file, &mut content)
            .await
            .map_err(|e| {
                err_with_context!(e, "Reading timeblocks/{}.json", day.format("%Y-%m-%d"))
            })?;
        if content.is_empty() {
            return Ok(vec![]);
        }
        let timeblocks: Vec<TimeBlock> = serde_json::from_str(&content).map_err(|e| {
            err_with_context!(
                e,
                "Deserializing timeblocks/{}.json",
                day.format("%Y-%m-%d")
            )
        })?;
        Ok(timeblocks)
    }

    pub async fn save(&self) -> Result<(), Error> {
        // Save to the end time day file.
        // If the day changed, find previous day records. If they exist, split the block in two and save them.
        let day = self.end_time.date_naive();
        let start_day = self.start_time.date_naive();
        let mut self_clone = self.clone();
        if day != start_day {
            let mut timeblocks = TimeBlock::get_day_timeblocks(start_day)
                .await
                .unwrap_or_default();
            // End at 11:59:59 of the start day
            let end_time = start_day
                .and_time(NaiveTime::from_hms_opt(23, 59, 59).ok_or(err_from_type!(
                    ErrorType::Chrono,
                    "Creating end time for {}",
                    start_day.format("%Y-%m-%d")
                ))?)
                .and_local_timezone(Local)
                .single()
                .ok_or(err_from_type!(
                    ErrorType::Chrono,
                    "No single time identifiable for end time for {}",
                    start_day.format("%Y-%m-%d")
                ))?;
            timeblocks.push(TimeBlock::new(
                self.start_time,
                end_time,
                self.block_type_id,
                self.title.clone(),
            ));
            let file_name = format!("timeblocks/{}.json", start_day.format("%Y-%m-%d"));
            let content = serde_json::to_string_pretty(&timeblocks).map_err(|e| {
                err_with_context!(
                    e,
                    "Serializing timeblocks/{}.json",
                    start_day.format("%Y-%m-%d")
                )
            })?;
            tokio::fs::write(file_name, content).await.map_err(|e| {
                err_with_context!(
                    e,
                    "Writing timeblocks/{}.json",
                    start_day.format("%Y-%m-%d")
                )
            })?;
            self_clone.start_time = day
                .and_time(NaiveTime::from_hms_opt(0, 0, 0).ok_or(err_from_type!(
                    ErrorType::Chrono,
                    "Creating start time for {}",
                    day.format("%Y-%m-%d")
                ))?)
                .and_local_timezone(Local)
                .single()
                .ok_or(err_from_type!(
                    ErrorType::Chrono,
                    "No single time identifiable for start time for {}",
                    day.format("%Y-%m-%d")
                ))?;
        }
        let mut timeblocks = TimeBlock::get_day_timeblocks(day).await.unwrap_or_default();
        timeblocks.push(self_clone);
        let file_name = format!("timeblocks/{}.json", day.format("%Y-%m-%d"));
        let contents = serde_json::to_string_pretty(&timeblocks).map_err(|e| {
            err_with_context!(e, "Serializing timeblocks/{}.json", day.format("%Y-%m-%d"))
        })?;
        tokio::fs::write(file_name, contents).await.map_err(|e| {
            err_with_context!(e, "Writing timeblocks/{}.json", day.format("%Y-%m-%d"))
        })?;
        Ok(())
    }

    pub async fn split_timeblock(split_time_block_query: SplitTimeBlockQuery) -> Result<(), Error> {
        let day = split_time_block_query.start_time.date_naive();
        let mut timeblocks = TimeBlock::get_day_timeblocks(day).await?;
        let target_block = timeblocks
            .iter()
            .find(|b| {
                b.start_time == split_time_block_query.start_time
                    && b.end_time == split_time_block_query.end_time
            })
            .ok_or(err_from_type!(
                ErrorType::NotFound,
                "Time block not found from {} to {}",
                split_time_block_query
                    .start_time
                    .format("%Y-%m-%d %H:%M:%S"),
                split_time_block_query.end_time.format("%Y-%m-%d %H:%M:%S")
            ))?;
        let block_idx = timeblocks
            .iter()
            .position(|b| b == target_block)
            .ok_or(err_from_type!(
                ErrorType::InternalRustError,
                "Time block not found from {} to {} after finding it",
                split_time_block_query
                    .start_time
                    .format("%Y-%m-%d %H:%M:%S"),
                split_time_block_query.end_time.format("%Y-%m-%d %H:%M:%S")
            ))?;
        let before_block = TimeBlock::new(
            split_time_block_query.start_time,
            split_time_block_query.split_time,
            split_time_block_query.before_block_type_id,
            split_time_block_query.before_title,
        );
        let after_block = TimeBlock::new(
            split_time_block_query.split_time,
            split_time_block_query.end_time,
            split_time_block_query.after_block_type_id,
            split_time_block_query.after_title,
        );

        timeblocks.remove(block_idx);
        timeblocks.insert(block_idx, before_block);
        timeblocks.insert(block_idx + 1, after_block);

        let file_name = format!("timeblocks/{}.json", day.format("%Y-%m-%d"));
        let content = serde_json::to_string_pretty(&timeblocks).map_err(|e| {
            err_with_context!(e, "Serializing timeblocks/{}.json", day.format("%Y-%m-%d"))
        })?;
        tokio::fs::write(file_name, content).await.map_err(|e| {
            err_with_context!(e, "Writing timeblocks/{}.json", day.format("%Y-%m-%d"))
        })?;
        Ok(())
    }
}
