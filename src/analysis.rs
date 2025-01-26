use chrono::{DateTime, Local, NaiveDate, TimeDelta};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, time::Duration};

use crate::{
    blocktype::BlockType,
    err_from_type, err_with_context,
    error::{Error, ErrorType},
    timeblock::TimeBlock,
};

#[derive(Serialize, Deserialize)]
pub struct AnalysisQuery {
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

#[derive(Serialize, Deserialize)]
pub struct Trend {
    pub day: NaiveDate,
    pub time_spent: Duration,
    pub block_type_id: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Analysis {
    pub percentages: Vec<f32>,
    pub trends: Vec<Trend>,
    pub blocktypes: Vec<BlockType>,
}

impl Analysis {
    pub async fn get_analysis_data(
        data_dir: &Path,
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
    ) -> Result<Analysis, Error> {
        let start_time = start_time.date_naive();
        let end_time = end_time.date_naive();
        let mut blocktypes = BlockType::load(data_dir).await?;
        blocktypes.sort_by(|a, b| a.id.cmp(&b.id));

        let mut iter_time = start_time;
        let mut durations: HashMap<u8, Duration> = HashMap::new();
        let mut trends: Vec<Trend> = Vec::new();

        while iter_time <= end_time {
            let blocks = TimeBlock::get_day_timeblocks(data_dir, iter_time).await?;
            for blocktype in &blocktypes {
                let mut time_spent = Duration::from_secs(0);
                for block in &blocks {
                    if block.block_type_id == blocktype.id {
                        time_spent += block
                            .duration()
                            .to_std()
                            .map_err(|e| err_with_context!(e, "Converting duration to std"))?
                    }
                }

                let trend = Trend {
                    day: iter_time,
                    time_spent,
                    block_type_id: blocktype.id,
                };
                trends.push(trend);
                if durations.contains_key(&blocktype.id) {
                    durations.insert(blocktype.id, durations[&blocktype.id] + time_spent);
                } else {
                    durations.insert(blocktype.id, time_spent);
                }
            }

            iter_time += TimeDelta::new(24 * 60 * 60, 0).ok_or(err_from_type!(
                ErrorType::Chrono,
                "Creating next day's time"
            ))?;
        }

        let mut total_time = Duration::from_secs(0);
        for duration in durations.values() {
            total_time += *duration;
        }

        let mut percentage_map: HashMap<u8, f32> = HashMap::new();
        for (blocktype_id, duration) in &durations {
            let percentage = (duration.as_secs() as f32) / (total_time.as_secs() as f32);
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
            blocktypes,
        })
    }
}
