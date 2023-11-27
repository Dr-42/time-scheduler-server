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

    pub fn get_day_timeblocks(&self) -> Result<Vec<Self>> {
        if !std::path::Path::new("timeblocks").exists() {
            std::fs::create_dir("timeblocks")?;
        }
        if !std::path::Path::new(&self.endTime.filename()).exists() {
            std::fs::File::create(self.endTime.filename())?;
            return Ok(Vec::new());
        }
        let timeblocks =
            serde_json::from_str::<Vec<Self>>(&std::fs::read_to_string(self.endTime.filename())?)?;
        Ok(timeblocks)
    }

    pub fn save(&self) -> Result<()> {
        let mut timeblocks = self.get_day_timeblocks()?;
        if self.check_overlaps(&timeblocks) {
            return Err("Timeblock overlaps with another timeblock".into());
        }
        timeblocks.push(self.clone());
        serde_json::to_writer_pretty(std::fs::File::create(self.endTime.filename())?, &timeblocks)?;
        Ok(())
    }
}
