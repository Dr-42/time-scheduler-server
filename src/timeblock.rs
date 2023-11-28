use crate::{duration::Duration, time::Time, Result};
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeBlock {
    pub startTime: Time,
    pub endTime: Time,
    pub blockTypeId: u8,
    pub title: String,
}

impl TimeBlock {
    pub fn duration(&self) -> crate::duration::Duration {
        self.startTime.time_span(&self.endTime)
    }

    pub fn check_overlap(&self, other: &Self) -> bool {
        if self.startTime.before(&other.startTime) {
            if self.endTime.before(&other.startTime) {
                return false;
            }
        } else {
            return !other.endTime.before(&self.startTime);
        }
        true
    }

    pub fn check_overlaps(&self, others: &[Self]) -> bool {
        for other in others {
            if self.check_overlap(other) {
                println!("Overlap: {:?} {:?}", self, other);
                return true;
            }
        }
        false
    }

    pub fn get_day_timeblocks(time: &Time) -> Result<Vec<Self>> {
        if !std::path::Path::new("timeblocks").exists() {
            std::fs::create_dir("timeblocks")?;
        }
        if !std::path::Path::new(&time.filename()).exists() {
            std::fs::File::create(time.filename())?;
            return Ok(Vec::new());
        }
        let file_contents = std::fs::read_to_string(time.filename());
        if file_contents.is_err() {
            return Ok(Vec::new());
        }
        let timeblocks = serde_json::from_str::<Vec<Self>>(file_contents.unwrap().as_str());
        if timeblocks.is_err() {
            return Ok(Vec::new());
        }
        let timeblocks = timeblocks.unwrap();
        Ok(timeblocks)
    }

    pub fn save(&mut self) -> Result<()> {
        if !std::path::Path::new("timeblocks").exists() {
            std::fs::create_dir("timeblocks")?;
        }
        if !std::path::Path::new(&self.endTime.filename()).exists() {
            std::fs::File::create(self.endTime.filename())?;
        }
        // Check if the start time is 1945/1/1 1:1:1
        // If so, set the start time to the endtime of the previous day's last time block
        if self.startTime == Time::new(1945, 1, 1, 1, 1, 1).unwrap() {
            let prev_day = self.endTime - Duration::from_seconds(24 * 60 * 60);
            let prev_day_timeblocks = Self::get_day_timeblocks(&prev_day)?;
            if prev_day_timeblocks.is_empty() {
                self.startTime = self.endTime;
            } else {
                self.startTime = self.endTime;
                self.startTime.hour = 0;
                self.startTime.minute = 0;
                self.startTime.second = 0;

                let last_timeblock = prev_day_timeblocks.last().unwrap();
                let mut prev_day_end_time = last_timeblock.endTime;
                prev_day_end_time.hour = 23;
                prev_day_end_time.minute = 59;
                prev_day_end_time.second = 59;
                let mut prev_day_block = Self {
                    startTime: prev_day_timeblocks.last().unwrap().endTime,
                    endTime: prev_day_end_time,
                    blockTypeId: self.blockTypeId,
                    title: self.title.clone(),
                };
                prev_day_block.save()?;
            }
        }
        let mut timeblocks = Self::get_day_timeblocks(&self.endTime)?;
        if timeblocks.is_empty() {
            let mut new_day_time = self.endTime;
            new_day_time.hour = 0;
            new_day_time.minute = 0;
            new_day_time.second = 0;
            let new_day_block = Self {
                startTime: new_day_time,
                endTime: new_day_time,
                blockTypeId: 0,
                title: "New Day".into(),
            };
            timeblocks.push(new_day_block);
            timeblocks.push(self.clone());
            serde_json::to_writer_pretty(
                std::fs::File::create(self.endTime.filename())?,
                &timeblocks,
            )?;
            return Ok(());
        }
        if self.check_overlaps(&timeblocks) {
            return Err("Timeblock overlaps with another timeblock".into());
        }
        timeblocks.push(self.clone());
        serde_json::to_writer_pretty(std::fs::File::create(self.endTime.filename())?, &timeblocks)?;
        Ok(())
    }
}
