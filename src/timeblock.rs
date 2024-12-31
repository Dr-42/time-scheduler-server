use std::path::Path;

use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum TimeBlockError {
    Serde(serde_json::Error),
    Tokio(tokio::io::Error),
    Chrono,
}

impl std::fmt::Display for TimeBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeBlockError::Serde(e) => write!(f, "Serde error: {}", e),
            TimeBlockError::Tokio(e) => write!(f, "Tokio error: {}", e),
            TimeBlockError::Chrono => write!(f, "Chrono error"),
        }
    }
}

impl From<serde_json::Error> for TimeBlockError {
    fn from(e: serde_json::Error) -> Self {
        TimeBlockError::Serde(e)
    }
}

impl From<tokio::io::Error> for TimeBlockError {
    fn from(e: tokio::io::Error) -> Self {
        TimeBlockError::Tokio(e)
    }
}

impl From<chrono::ParseError> for TimeBlockError {
    fn from(_e: chrono::ParseError) -> Self {
        TimeBlockError::Chrono
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeBlock {
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub block_type_id: u8,
    pub title: String,
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

    pub async fn get_day_timeblocks(day: NaiveDate) -> Result<Vec<TimeBlock>, TimeBlockError> {
        if !Path::new("timeblocks").exists() {
            std::fs::create_dir("timeblocks")?;
        }

        let file_name = format!("timeblocks/{}.json", day.format("%Y-%m-%d"));
        let file = tokio::fs::File::open(file_name).await;
        if let Err(e) = &file {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok(vec![]);
            }
        }
        let mut file = file?;
        let mut content = String::new();
        tokio::io::AsyncReadExt::read_to_string(&mut file, &mut content).await?;
        if content.is_empty() {
            return Ok(vec![]);
        }
        let timeblocks: Vec<TimeBlock> = serde_json::from_str(&content)?;
        Ok(timeblocks)
    }

    pub async fn save(&self) -> Result<(), TimeBlockError> {
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
            let end_time = DateTime::from_naive_utc_and_offset(
                start_day
                    .and_time(NaiveTime::from_hms_opt(11, 59, 59).ok_or(TimeBlockError::Chrono)?),
                *Local::now().offset(),
            );
            timeblocks.push(TimeBlock::new(
                self.start_time,
                end_time,
                self.block_type_id,
                self.title.clone(),
            ));
            let file_name = format!("timeblocks/{}.json", start_day.format("%Y-%m-%d"));
            let mut file = if Path::new(&file_name).exists() {
                tokio::fs::File::open(file_name).await?
            } else {
                tokio::fs::File::create(file_name).await?
            };
            let content = serde_json::to_string_pretty(&timeblocks)?;
            tokio::io::AsyncWriteExt::write_all(&mut file, content.as_bytes()).await?;
            self_clone.start_time = DateTime::from_naive_utc_and_offset(
                day.and_time(NaiveTime::from_hms_opt(0, 0, 0).ok_or(TimeBlockError::Chrono)?),
                *Local::now().offset(),
            )
        }
        let mut timeblocks = TimeBlock::get_day_timeblocks(day).await.unwrap_or_default();
        timeblocks.push(self_clone);
        let file_name = format!("timeblocks/{}.json", day.format("%Y-%m-%d"));
        let contents = serde_json::to_string_pretty(&timeblocks)?;
        tokio::fs::write(file_name, contents).await?;
        Ok(())
    }
}
