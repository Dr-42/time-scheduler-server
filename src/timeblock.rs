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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdjustTimeBlockQuery {
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    new_start_time: DateTime<Local>,
    new_end_time: DateTime<Local>,
    title: String,
    block_type_id: u8,
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

    pub async fn get_day_timeblocks(
        data_dir: &Path,
        day: NaiveDate,
    ) -> Result<Vec<TimeBlock>, Error> {
        let time_blocks_dir = data_dir.join("timeblocks");
        if !time_blocks_dir.exists() {
            tokio::fs::create_dir(&time_blocks_dir)
                .await
                .map_err(|e| err_with_context!(e, "Creating timeblocks directory"))?;
        }

        let file_name = time_blocks_dir.join(format!("{}.json", day.format("%Y-%m-%d")));
        let file = tokio::fs::File::open(&file_name).await;
        if let Err(e) = &file {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok(vec![]);
            }
        }
        let mut file = file.map_err(|e| err_with_context!(e, "Opening {}", file_name.display()))?;
        let mut content = String::new();
        tokio::io::AsyncReadExt::read_to_string(&mut file, &mut content)
            .await
            .map_err(|e| err_with_context!(e, "Reading {}", file_name.display()))?;
        if content.is_empty() {
            return Ok(vec![]);
        }
        let timeblocks: Vec<TimeBlock> = serde_json::from_str(&content)
            .map_err(|e| err_with_context!(e, "Deserializing {}", file_name.display()))?;
        Ok(timeblocks)
    }

    pub async fn save(&self, data_dir: &Path) -> Result<(), Error> {
        // Save to the end time day file.
        // If the day changed, find previous day records. If they exist, split the block in two and save them.
        let day = self.end_time.date_naive();
        let start_day = self.start_time.date_naive();
        let mut self_clone = self.clone();
        if day != start_day {
            let mut timeblocks = TimeBlock::get_day_timeblocks(data_dir, start_day)
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
            //let file_name = format!("timeblocks/{}.json", start_day.format("%Y-%m-%d"));
            let file_name = data_dir
                .join("timeblocks")
                .join(format!("{}.json", start_day.format("%Y-%m-%d")));
            let content = serde_json::to_string_pretty(&timeblocks)
                .map_err(|e| err_with_context!(e, "Serializing {}", file_name.display()))?;
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
        let mut timeblocks = TimeBlock::get_day_timeblocks(data_dir, day)
            .await
            .unwrap_or_default();
        timeblocks.push(self_clone);
        let file_name = data_dir
            .join("timeblocks")
            .join(format!("{}.json", day.format("%Y-%m-%d")));
        let contents = serde_json::to_string_pretty(&timeblocks)
            .map_err(|e| err_with_context!(e, "Serializing {}", file_name.display()))?;
        tokio::fs::write(&file_name, contents)
            .await
            .map_err(|e| err_with_context!(e, "Writing timeblocks {}", file_name.display()))?;
        Ok(())
    }

    pub async fn split_timeblock(
        data_dir: &Path,
        split_time_block_query: SplitTimeBlockQuery,
    ) -> Result<(), Error> {
        let day = split_time_block_query.start_time.date_naive();
        let mut timeblocks = TimeBlock::get_day_timeblocks(data_dir, day).await?;
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

        let file_name = data_dir
            .join("timeblocks")
            .join(format!("{}.json", day.format("%Y-%m-%d")));
        let content = serde_json::to_string_pretty(&timeblocks)
            .map_err(|e| err_with_context!(e, "Serializing {}", file_name.display()))?;
        tokio::fs::write(&file_name, content)
            .await
            .map_err(|e| err_with_context!(e, "Writing {}", file_name.display()))?;
        Ok(())
    }

    pub async fn adjust_timeblock(
        data_dir: &Path,
        adjust_time_block_query: AdjustTimeBlockQuery,
    ) -> Result<(), Error> {
        let day = adjust_time_block_query.start_time.date_naive();
        let mut timeblocks = TimeBlock::get_day_timeblocks(data_dir, day).await?;
        let target_block = timeblocks
            .iter()
            .find(|b| {
                b.start_time == adjust_time_block_query.start_time
                    && b.end_time == adjust_time_block_query.end_time
            })
            .ok_or(err_from_type!(
                ErrorType::NotFound,
                "Time block not found from {} to {}",
                adjust_time_block_query
                    .start_time
                    .format("%Y-%m-%d %H:%M:%S"),
                adjust_time_block_query.end_time.format("%Y-%m-%d %H:%M:%S")
            ))?;
        let block_idx = timeblocks
            .iter()
            .position(|b| b == target_block)
            .ok_or(err_from_type!(
                ErrorType::InternalRustError,
                "Time block not found from {} to {} after finding it",
                adjust_time_block_query
                    .start_time
                    .format("%Y-%m-%d %H:%M:%S"),
                adjust_time_block_query.end_time.format("%Y-%m-%d %H:%M:%S")
            ))?;

        if let Some(pre_block) = timeblocks.get_mut(block_idx - 1) {
            pre_block.end_time = adjust_time_block_query.new_start_time;
        }
        if let Some(post_block) = timeblocks.get_mut(block_idx + 1) {
            post_block.start_time = adjust_time_block_query.new_end_time;
        }
        let new_block = TimeBlock::new(
            adjust_time_block_query.new_start_time,
            adjust_time_block_query.new_end_time,
            adjust_time_block_query.block_type_id,
            adjust_time_block_query.title,
        );
        timeblocks.remove(block_idx);
        timeblocks.insert(block_idx, new_block);

        let file_name = data_dir
            .join("timeblocks")
            .join(format!("{}.json", day.format("%Y-%m-%d")));
        let content = serde_json::to_string_pretty(&timeblocks)
            .map_err(|e| err_with_context!(e, "Serializing {}", file_name.display()))?;
        tokio::fs::write(&file_name, &content)
            .await
            .map_err(|e| err_with_context!(e, "Writing {}", file_name.display()))?;
        Ok(())
    }
}
