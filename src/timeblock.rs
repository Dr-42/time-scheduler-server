use crate::{time::Time, Result};
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
        let timeblocks =
            serde_json::from_str::<Vec<Self>>(&std::fs::read_to_string(time.filename())?)?;
        Ok(timeblocks)
    }

    pub fn save(&self) -> Result<()> {
        // Check if time block started earlier than today's date
        if self.startTime.day < self.endTime.day {
            let mut prev_day_block = self.clone();
            prev_day_block.endTime.day = prev_day_block.startTime.day;
            prev_day_block.endTime.hour = 23;
            prev_day_block.endTime.minute = 59;
            prev_day_block.endTime.second = 59;
            prev_day_block.save()?;

            let mut next_day_block = self.clone();
            next_day_block.startTime.day = next_day_block.endTime.day;
            next_day_block.startTime.hour = 0;
            next_day_block.startTime.minute = 0;
            next_day_block.startTime.second = 0;
            next_day_block.save()?;
            return Ok(());
        }
        // Check if the start time is 1945/1/1 1:1:1
        // If so, set the start time to the endtime of the previous day's last time block
        if self.startTime == Time::new(1945, 1, 1, 1, 1, 1).unwrap() {
            let mut prev_day_block = self.clone();
            prev_day_block.endTime = prev_day_block.startTime;
            prev_day_block.save()?;
            return Ok(());
        }
        let mut timeblocks = Self::get_day_timeblocks(&self.endTime)?;
        if self.check_overlaps(&timeblocks) {
            return Err("Timeblock overlaps with another timeblock".into());
        }
        timeblocks.push(self.clone());
        serde_json::to_writer_pretty(std::fs::File::create(self.endTime.filename())?, &timeblocks)?;
        Ok(())
    }
}
