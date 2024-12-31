#![allow(clippy::unwrap_used, clippy::expect_used)]
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{fs::create_dir_all, sync::Mutex};

use crate::{currentblock::CurrentBlock, timeblock::TimeBlock};

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct OldTime {
    pub year: u32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OldTimeBlock {
    pub startTime: OldTime,
    pub endTime: OldTime,
    pub blockTypeId: u8,
    pub title: String,
}

impl From<OldTimeBlock> for TimeBlock {
    fn from(value: OldTimeBlock) -> Self {
        // Remove " from either sides of the title if they exist
        let new_title = value.title.trim_matches('"').to_string();
        Self {
            start_time: value.startTime.into(),
            end_time: value.endTime.into(),
            block_type_id: value.blockTypeId,
            title: new_title,
        }
    }
}

impl From<OldTime> for DateTime<Local> {
    fn from(value: OldTime) -> Self {
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(value.year as i32, value.month as u32, value.day as u32)
                .unwrap(),
            NaiveTime::from_hms_opt(value.hour as u32, value.minute as u32, value.second as u32)
                .unwrap(),
        )
        .and_local_timezone(Local)
        .unwrap()
    }
}

pub async fn migrate(overwrite: bool) {
    migrate_folder("./timeblocks", overwrite).await;
    migrate_current(overwrite).await.unwrap();
    if !overwrite {
        std::fs::copy("./blocktypes.json", "migrations/blocktypes.json").unwrap();
    }
}

pub async fn migrate_folder(folder_path: &str, overwrite: bool) {
    let files = std::fs::read_dir(folder_path).unwrap();
    let mut paths = files.map(|f| f.unwrap().path()).collect::<Vec<_>>();
    paths.sort();
    if !Path::new("./migrations").exists() && !overwrite {
        create_dir_all("migrations/timeblocks").await.unwrap();
    }
    let last_block_append_count = Arc::new(Mutex::new(0));
    let mut errors = Vec::new();
    let mut done = 0;
    for file in paths {
        let handle = migrate_file(file.clone(), overwrite, last_block_append_count.clone()).await;
        if let Err(err) = handle {
            errors.push(format!("Error migrating file {}: {}", file.display(), err));
        }
        done += 1;
    }
    if !errors.is_empty() {
        let mut error_file = std::fs::File::create("errors.txt").unwrap();
        let error_len = errors.len();
        for error in errors {
            error_file.write_all(error.as_bytes()).unwrap();
        }
        println!("Encountered {} errors", error_len);
    } else {
        println!("No errors encountered");
    }
    println!("Migrated {} files", done);
    let last_block_append_count = last_block_append_count.lock().await;
    if *last_block_append_count > 0 {
        println!(
            "Appended {} blocks to the last block of the day",
            *last_block_append_count
        );
    }
}

pub async fn migrate_file(
    file_path: PathBuf,
    overwrite: bool,
    last_block_append_count: Arc<Mutex<u64>>,
) -> Result<(), std::io::Error> {
    let file_name = Path::new(&file_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let output_path = if !overwrite {
        format!("migrations/timeblocks/{}", file_name)
    } else {
        //file_name.to_string()
        format!("timeblocks/{}", file_name)
    };
    let input: Result<Vec<OldTimeBlock>, serde_json::Error> =
        serde_json::from_str(&std::fs::read_to_string(file_path)?);
    let input = match input {
        Ok(input) => input,
        Err(err) => {
            println!("Error parsing file {}: {}", file_name, err);
            return Ok(());
        }
    };
    let mut output_content = input
        .into_iter()
        .filter_map(|tb| {
            if tb.blockTypeId == 0 {
                return None;
            }
            Some(tb.into())
        })
        .collect::<Vec<TimeBlock>>();

    // Check the last time block, if it doesn't end at 11:59:59, then open the next day and copy
    // it's first non zero block to the end of the current day

    if let Some(last_block) = output_content.last() {
        if last_block.end_time.time().hour() != 23
            || last_block.end_time.time().minute() != 59
            || last_block.end_time.time().second() != 59
        {
            let next_day = last_block.end_time.date_naive() + chrono::Duration::days(1);
            let next_day_file_name = format!("./timeblocks/{}.json", next_day.format("%Y-%m-%d"));
            if Path::new(&next_day_file_name).exists() {
                let next_day_file = std::fs::read_to_string(&next_day_file_name).unwrap();
                let next_day_blocks: Result<Vec<OldTimeBlock>, serde_json::Error> =
                    serde_json::from_str(&next_day_file);
                if let Err(err) = next_day_blocks {
                    println!("Error parsing file {}: {}", next_day_file_name, err);
                    return Ok(());
                }
                let next_day_blocks = next_day_blocks.unwrap();
                let next_day_first_block = next_day_blocks
                    .into_iter()
                    .find(|tb| tb.blockTypeId != 0)
                    .unwrap();
                let proto_block = TimeBlock::from(next_day_first_block);
                let end_time = last_block
                    .end_time
                    .with_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap())
                    .unwrap();
                let new_last_block = TimeBlock {
                    start_time: last_block.end_time,
                    end_time,
                    block_type_id: proto_block.block_type_id,
                    title: proto_block.title,
                };
                output_content.push(new_last_block);
                let mut last_block_append_count = last_block_append_count.lock().await;
                *last_block_append_count += 1;
            }
        }
    }

    let output = serde_json::to_string_pretty(&output_content).unwrap();
    std::fs::write(output_path, output).unwrap();
    Ok(())
}

pub async fn migrate_current(overwrite: bool) -> Result<(), std::io::Error> {
    let block_name_file = "./currentblockname.txt";
    let block_type_file = "./currentblocktype.txt";

    let output_file = if !overwrite {
        "migrations/currentblock.json"
    } else {
        "currentblock.json"
    };

    let block_name = std::fs::read_to_string(block_name_file)?;
    let block_type = std::fs::read_to_string(block_type_file)?;

    let output = serde_json::to_string_pretty(&CurrentBlock {
        current_block_name: block_name,
        block_type_id: block_type.parse().unwrap(),
    })?;

    std::fs::write(output_file, output)?;

    if overwrite {
        std::fs::remove_file(block_name_file)?;
        std::fs::remove_file(block_type_file)?;
    }
    Ok(())
}
